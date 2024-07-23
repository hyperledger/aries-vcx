package org.hyperledger.ariesvcx

import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
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
    val connectionCompleted: Boolean = false,
    val offerReceived: Boolean = false
)

class AppDemoController : ViewModel() {
    private val httpClient = OkHttpClient()

    private var profile: ProfileHolder? = null
    private var connection: Connection? = null
    private var holder: Holder? = null

    private var onConnectionComplete: (connection: Connection) -> Unit = {}
    private var onOfferReceived: () -> Unit = {}

    // Expose screen UI state
    private val _state = MutableStateFlow(AppUiState())
    val states: StateFlow<AppUiState> = _state.asStateFlow()

    fun getHolder (): Holder? {
        return holder
    }

    fun getProfileHolder (): ProfileHolder {
        return profile!!
    }

    private val walletConfig = AskarWalletConfig(
        dbUrl = "sqlite://:memory:",
        keyMethod = KeyMethod.DeriveKey(AskarKdfMethod.Argon2i(ArgonLevel.INTERACTIVE)),
        passKey = "test",
        profile = "profile")

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
                Log.d("RELAY RESPONSE", "RELAY RESPONDED WITH 200")
                val message = relayResponse.body!!.string()
                Log.d("MESSAGE", "awaitConnectionCompletion: $message")
                Log.d("PROFILE", "profile: ${profile.toString()}")
                val unpackedMessage = unpackMessage(
                    profile!!,
                    message
                )

                Log.d("AppDemoController", unpackedMessage.message)
                connection!!.handleResponse(profile!!, unpackedMessage.message)
                connection!!.sendAck(profile!!)

                Log.d("AppDemoController", "connection state: ${connection!!.getState()}")

                _state.update { it.copy(connectionCompleted = true) }
                onConnectionComplete.invoke(connection!!)
                break
            }
        }
    }

    fun subscribeToConnectionComplete(onComplete: (connection: Connection) -> Unit) {
        onConnectionComplete = onComplete
    }

    suspend fun processOfferRequest() {
        withContext(Dispatchers.IO) {
            holder?.prepareCredentialRequest(profile!!, "4xE68b6S5VRFrKMMG1U95M")
            Log.d("HOLDER", "processOfferRequest: ${holder?.getState()}")
            val message = holder?.getMsgCredentialRequest()
            connection?.sendMessage(profile!!, message!!)
        }
    }

    suspend fun awaitCredentialPolling() {
        val pollRelayRequest = Request.Builder()
            .url("$BASE_RELAY_ENDPOINT/pop_user_message/$RELAY_USER_ID")
            .build()
        while(true) {
            delay(2000)
            val relayResponse = httpClient.newCall(pollRelayRequest).await()
            if (relayResponse.code == 200) {
                val message = relayResponse.body!!.string()

                val unpackedMessage = unpackMessage(
                    profile!!,
                    message
                )

                Log.d("MESSAGE", "$unpackedMessage.message")

                if (!_state.value.offerReceived) {
                    Log.d("OFFER", "awaitCredentialPolling: received OFFER")
                    holder = createFromOffer("", unpackedMessage.message)

                    _state.update { it.copy(offerReceived = true) }
                    onOfferReceived.invoke()
                } else {
                    Log.d("CREDENTIAL", "awaitCredentialPolling: received CREDENTIAL")
                    holder?.processCredential(profile!!, unpackedMessage.message)

                    _state.update { it.copy(offerReceived = false) }
                }
            }
        }
    }
}
