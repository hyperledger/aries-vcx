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
import androidx.compose.runtime.collectAsState
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
    demoController: AppDemoController,
    navController: NavHostController,
) {

    val demoState by demoController.states.collectAsState()

    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    demoController.subscribeToConnectionComplete { newConn ->
        scope.launch(Dispatchers.Main) {
            Toast.makeText(
                context,
                "New Connection Created",
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Button(
            enabled = (!demoState.profileReady),
            onClick = {
                scope.launch {
                    demoController.setupProfile()
                    withContext(Dispatchers.Main) {
                        Toast.makeText(
                            context,
                            "New Profile Created",
                            Toast.LENGTH_SHORT
                        ).show()
                    }
                }
            }) {
            Text(text = "New Indy Profile")
        }
        Button(enabled = (demoState.profileReady && !demoState.connectionInvitationReceived),
            onClick = {
                navController.navigate(Destination.QRScan.route)
            }) {
            Text(text = "Scan QR Code")
        }
        Button(enabled = (demoState.connectionCompleted),
            onClick = {
            }) {
            Text(text = "Receive a credential")
        }
    }
}
