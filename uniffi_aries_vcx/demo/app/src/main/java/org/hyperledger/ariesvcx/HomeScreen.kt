package org.hyperledger.ariesvcx

import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.navigation.NavHostController
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
fun HomeScreen(navController: NavHostController, setProfileHolder: (ProfileHolder) -> Unit, profileHolder: ProfileHolder?, connection: Connection?) {

    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    val walletConfigState = WalletConfig(
        walletName = "test_create_wallet_add_uuid_here",
        walletKey = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY",
        walletKeyDerivation = "RAW",
        walletType = null,
        storageConfig = null,
        storageCredentials = null,
        rekey = null,
        rekeyDerivationMethod = null
    )

    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Button(
            enabled = (connection == null),
            onClick = {
            scope.launch(Dispatchers.IO) {
                val profile = newIndyProfile(walletConfigState)
                setProfileHolder(profile)
                withContext(Dispatchers.Main) {
                    Toast.makeText(context, profile.toString(), Toast.LENGTH_SHORT).show()
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
