package org.hyperledger.ariesvcx

import android.os.Bundle
import android.system.Os
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import org.hyperledger.ariesvcx.ui.theme.DemoTheme

sealed class Destination(val route: String) {
    object Home: Destination("home")
    object QRScan: Destination("scan")
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Os.setenv("EXTERNAL_STORAGE", this.filesDir.absolutePath, true);
        setContent {
            DemoTheme {
                Surface (
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    val navController = rememberNavController()
                    NavigationAppHost(navController = navController)

                }
            }
        }
    }
}

@Composable
fun NavigationAppHost(navController: NavHostController) {
    NavHost(navController = navController, startDestination = "home") {
        composable(Destination.Home.route) {
            HomeScreen(navController)
        }
        composable(Destination.QRScan.route) {
            ScanScreen()
        }
    }
}
