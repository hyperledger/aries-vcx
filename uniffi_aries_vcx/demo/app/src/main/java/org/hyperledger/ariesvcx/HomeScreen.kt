package org.hyperledger.ariesvcx

import android.os.Handler
import android.os.Looper
import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import kotlinx.coroutines.launch

@Composable
fun HomeScreen(navController: NavHostController, setProfileHolder: (ProfileHolder) -> Unit) {

    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    var walletConfigState by remember {
        mutableStateOf(WalletConfig(
            walletName = "test_create_wallet_add_uuid_here",
            walletKey = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY",
            walletKeyDerivation = "RAW",
            walletType = null,
            storageConfig = null,
            storageCredentials = null,
            rekey = null,
            rekeyDerivationMethod = null
        ))
    }

    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Button(onClick = {
            scope.launch(Dispatchers.IO) {
                val profile = newIndyProfile(walletConfigState)
                setProfileHolder(profile)
                Handler(Looper.getMainLooper()).post {
                    Toast.makeText(context, profile.toString(), Toast.LENGTH_SHORT).show()
                }
            }
        }) {
            Text(text = "New Indy Profile")
        }
        Button(onClick = {
            navController.navigate(Destination.QRScan.route)
        }) {
            Text(text = "Scan QR Code")
        }
    }
}
