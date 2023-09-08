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
import androidx.compose.ui.res.stringResource
import androidx.core.content.res.TypedArrayUtils
import androidx.navigation.NavHostController
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.Call
import okhttp3.Callback
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import java.io.IOException


@Composable
fun HomeScreen(
    navController: NavHostController,
    setProfileHolder: (ProfileHolder) -> Unit,
    profileHolder: ProfileHolder?,
    connection: Connection?,
    walletConfig: WalletConfig,
    requested: Boolean,
    httpClient: OkHttpClient
) {

    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    val TAG = "HomeScreen"

    var intercept by remember {
        mutableStateOf(true)
    }

    val serviceEndpoint = stringResource(R.string.serviceEndpoint)
    val request = Request.Builder()
        .url("$serviceEndpoint/pop_user_message/${walletConfig.walletKey}")
        .build()


    LaunchedEffect(true) {
        while (intercept && requested) {
            httpClient.newCall(request).enqueue(object : Callback {
                override fun onFailure(call: Call, e: IOException) {
                    Log.d(TAG, "onFailure: ${e.printStackTrace()}")
                }

                override fun onResponse(call: Call, response: Response) {
                    response.use {
                        if (response.code == 200) {
                            val message = response.body!!.string()

                            val unpackedMessage = unpackMessageAndReturnMessage(
                                profileHolder!!,
                                message
                            )

                            connection?.handleResponse(profileHolder, unpackedMessage)
                            intercept = false
                        }
                    }
                }
            })
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
