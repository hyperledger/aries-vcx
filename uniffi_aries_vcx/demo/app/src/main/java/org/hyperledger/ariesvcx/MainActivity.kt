package org.hyperledger.ariesvcx

import android.os.Bundle
import android.system.Os
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import okhttp3.OkHttpClient
import org.hyperledger.ariesvcx.ui.theme.DemoTheme


sealed class Destination(val route: String) {
    object Home : Destination("home")
    object QRScan : Destination("scan")
}

class MainActivity : ComponentActivity() {
    private var profile by mutableStateOf<ProfileHolder?>(null)
    private var connection by mutableStateOf<Connection?>(null)
    private var requested by mutableStateOf<Boolean>(false)
    private var httpClient = OkHttpClient()

    private val walletConfig = WalletConfig(
        walletName = "test_create_wallet_add_uuid_here",
        walletKey = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY",
        walletKeyDerivation = "RAW",
        walletType = null,
        storageConfig = null,
        storageCredentials = null,
        rekey = null,
        rekeyDerivationMethod = null
    )

    private fun setProfileHolder(profileHolder: ProfileHolder) {
        profile = profileHolder
        connection = createInvitee(profileHolder)
    }

    private fun setRequestedToTrue() {
        requested = true
    }


    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Os.setenv("EXTERNAL_STORAGE", this.filesDir.absolutePath, true)
        setContent {
            DemoTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    val navController = rememberNavController()
                    NavigationAppHost(
                        navController = navController,
                        setProfileHolder = { setProfileHolder(it) },
                        connection = connection,
                        profileHolder = profile,
                        walletConfig = walletConfig,
                        requested = requested,
                        setRequestedToTrue = { setRequestedToTrue() },
                        httpClient = httpClient
                    )
                }
            }
        }
    }
}

@Composable
fun NavigationAppHost(
    navController: NavHostController,
    setProfileHolder: (ProfileHolder) -> Unit,
    connection: Connection?,
    profileHolder: ProfileHolder?,
    walletConfig: WalletConfig,
    requested: Boolean,
    setRequestedToTrue: () -> Unit,
    httpClient: OkHttpClient,
) {
    NavHost(navController = navController, startDestination = "home") {
        composable(Destination.Home.route) {
            HomeScreen(
                navController = navController,
                setProfileHolder = setProfileHolder,
                profileHolder = profileHolder,
                connection = connection,
                walletConfig = walletConfig,
                requested = requested,
                httpClient = httpClient
            )
        }

        composable(Destination.QRScan.route) {
            ScanScreen(
                connection = connection!!,
                profileHolder = profileHolder!!,
                navController = navController,
                walletConfig = walletConfig,
                setRequestedToTrue = setRequestedToTrue
            )
        }
    }
}
