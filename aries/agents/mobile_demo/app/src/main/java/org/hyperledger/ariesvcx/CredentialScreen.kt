package org.hyperledger.ariesvcx

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavHostController
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

@Composable
fun CredentialScreen(
    demoController: AppDemoController,
    navController: NavHostController,
) {
    val jsonString = getCredentials(demoController.getProfileHolder());
    val gson = Gson()
    val typeToken = object : TypeToken<List<Credential>>() {}.type
    val credentialList: List<Credential> = gson.fromJson(jsonString, typeToken)
    val credentialAttrs = credentialList.map { it.attrs }

    Column(
        modifier = Modifier
            .fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text(
            text = "Received Credentials",
            textAlign = TextAlign.Center,
            fontSize = 30.sp
        )
        LazyColumn (verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(credentialAttrs) { attrs ->
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        attrs.forEach { (attributeName, attributeValue) ->
                            Text(
                                text = "$attributeName : $attributeValue",
                                textAlign = TextAlign.Center
                            )
                        }
                    }
                }
            }
        }
    }
}
