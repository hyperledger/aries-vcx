package org.hyperledger.ariesvcx

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import kotlinx.coroutines.launch

@Composable
fun HolderScreen(
    demoController: AppDemoController,
    navController: NavHostController,
) {
    val demoState by demoController.states.collectAsState()
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        demoController.awaitCredentialPolling()
    }

    if (demoState.offerReceived) {
        AlertDialog(
            onDismissRequest = { },
            title = { Text("Accept this credential?") },
            text = { Text(demoController.getHolder()?.getAttributes()!!) },
            confirmButton = {
                TextButton(onClick = {
                    scope.launch {
                        demoController.processOfferRequest()
                    }
                }) {
                    Text("Accept")
                }
            },
            dismissButton = {
                TextButton(onClick = { }) {
                    Text("Cancel")
                }
            },
        )
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(50.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(text = "Waiting for credential offer")
    }
}
