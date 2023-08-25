package org.hyperledger.ariesvcx

import android.os.Bundle
import android.system.Os
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Os.setenv("EXTERNAL_STORAGE", this.filesDir.absolutePath, true);
        setContent {
            var walletConfigState by remember {
                var walletConfig = WalletConfig(
                    walletName = "test_create_wallet_add_uuid_here",
                    walletKey = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY",
                    walletKeyDerivation = "RAW",
                    walletType = null,
                    storageConfig = null,
                    storageCredentials = null,
                    rekey = null,
                    rekeyDerivationMethod = null
                )
                mutableStateOf(walletConfig)
            }
            val snackbarHostState = remember { SnackbarHostState() }
            val scope = rememberCoroutineScope()
            Scaffold(Modifier.fillMaxSize(), snackbarHost = {
                SnackbarHost(snackbarHostState) { data ->
                    Snackbar(
                        modifier = Modifier
                            .padding(12.dp)
                    ) {
                        Text(data.visuals.message)
                    }
                }
            }) {
                Column(
                    modifier = Modifier
                        .padding(it)
                        .fillMaxSize(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center
                ){
                    Spacer(modifier = Modifier.height(16.dp))
                    Button(onClick = {
                        newIndyProfile(walletConfigState)
                        scope.launch {
                            snackbarHostState.showSnackbar("New Indy profile: ${walletConfigState}")
                        }
                    }) {
                        Text("Create new profile")
                    }
                }
            }
        }
    }
}
