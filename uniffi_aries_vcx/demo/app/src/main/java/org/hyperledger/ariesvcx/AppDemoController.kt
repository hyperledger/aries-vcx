package org.hyperledger.ariesvcx

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.hyperledger.ariesvcx.utils.await

data class AppUiState(
    val profileReady: Boolean = false,
    val connectionInvitationReceived: Boolean = false,
    val connectionCompleted: Boolean = false
)

class AppDemoController : ViewModel() {
    private val httpClient = OkHttpClient()

    private var profile: ProfileHolder? = null
    private var connection: Connection? = null

    private var onConnectionComplete: (connection: Connection) -> Unit = {}

    // Expose screen UI state
    private val _state = MutableStateFlow(AppUiState())
    val states: StateFlow<AppUiState> = _state.asStateFlow()

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

    suspend fun setupProfile(genesisFilePath: String) {
        withContext(Dispatchers.IO) {
            val newProfile = newIndyProfile(walletConfig, genesisFilePath)
            profile = newProfile
            connection = createInvitee(newProfile)
        }
        _state.update { current ->
            current.copy(profileReady = true)
        }
    }

    suspend fun acceptConnectionInvitation(invitation: String) {
        if (connection == null || profile == null) {
            throw Exception("Connection or Profile is null")
        }
        withContext(Dispatchers.IO) {
            connection!!.acceptInvitation(
                profile = profile!!,
                invitation = invitation
            )
            _state.update { it.copy(connectionInvitationReceived = true) }

            connection!!.sendRequest(
                profile!!,
                "$BASE_RELAY_ENDPOINT/send_user_message/$RELAY_USER_ID",
                emptyList()
            )

            // use viewmodel scope to finish off this work
            viewModelScope.launch(Dispatchers.IO) {
                awaitConnectionCompletion()
            }
        }
    }

    private suspend fun awaitConnectionCompletion() {
        val pollRelayRequest = Request.Builder()
            .url("$BASE_RELAY_ENDPOINT/pop_user_message/$RELAY_USER_ID")
            .build()
        while (true) {
            delay(500)
            val relayResponse = httpClient.newCall(pollRelayRequest).await()
            if (relayResponse.code == 200) {
                val message = relayResponse.body!!.string()

                val unpackedMessage = unpackMessage(
                    profile!!,
                    message
                )

                Log.d("AppDemoController", unpackedMessage.message)
                connection!!.handleResponse(profile!!, unpackedMessage.message)
                connection!!.sendAck(profile!!)

                Log.d("AppDemoController", "connection state: ${connection!!.getState()}")

                _state.update { it.copy(connectionCompleted = true) }
                onConnectionComplete(connection!!)
                break
            }
        }
    }

    fun subscribeToConnectionComplete(onComplete: (connection: Connection) -> Unit) {
        onConnectionComplete = onComplete
    }
}
