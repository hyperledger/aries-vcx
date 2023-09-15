package org.hyperledger.ariesvcx

import android.util.Log
import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.navigation.NavHostController
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.hyperledger.ariesvcx.utils.await


@Composable
fun HomeScreen(
    navController: NavHostController,
    setProfileHolder: (ProfileHolder) -> Unit,
    profileHolder: ProfileHolder?,
    connection: Connection?,
    walletConfig: WalletConfig,
    connectionRequestState: Boolean,
    httpClient: OkHttpClient
) {

    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    var flagKeepFetching by remember {
        mutableStateOf(true)
    }

    val request = Request.Builder()
        .url("$BASE_RELAY_ENDPOINT/pop_user_message/${walletConfig.walletKey}")
        .build()


    LaunchedEffect(true) {
        scope.launch(Dispatchers.IO) {
            while (flagKeepFetching && connectionRequestState) {
                delay(500)
                val response = httpClient.newCall(request).await()
                if (response.code == 200) {
                    val message = response.body!!.string()

                    val unpackedMessage = unpackMessage(
                        profileHolder!!,
                        message
                    )

                    Log.d("HOMESCREEN", "HomeScreen: ${unpackedMessage.message}")
                    connection?.handleResponse(profileHolder, unpackedMessage.message)
                    flagKeepFetching = false
                    connection?.sendAck(profileHolder)
                }
            }
        }
    }

    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Button(
            enabled = (connection == null),
            onClick = {
                scope.launch(Dispatchers.IO) {
                    val profile = newIndyProfile(walletConfig)
                    setProfileHolder(profile)
                    withContext(Dispatchers.Main) {
                        Toast.makeText(
                            context,
                            "New Profile Created: $profile",
                            Toast.LENGTH_SHORT
                        ).show()
                    }
                }
            }) {
            Text(text = "New Indy Profile")
        }
        Button(enabled = (profileHolder != null && connection != null),
            onClick = {
                navController.navigate(Destination.QRScan.route)
            }) {
            Text(text = "Scan QR Code")
        }
    }
}
