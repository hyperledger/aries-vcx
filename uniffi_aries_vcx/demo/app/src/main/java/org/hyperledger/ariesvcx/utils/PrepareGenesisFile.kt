package org.hyperledger.ariesvcx.utils

import android.content.Context
import android.util.Log
import org.hyperledger.ariesvcx.R
import java.io.BufferedWriter
import java.io.File
import java.io.FileWriter

fun prepareGenesisFile(context: Context): File {
    val file = File(context.filesDir, "genesis")
    if (!file.exists()) {
        val transactions = context.resources.openRawResource(R.raw.transactions).bufferedReader()
            .use { it.readText() }
        val bufferedWriter = BufferedWriter(FileWriter(file))
        bufferedWriter.write(transactions)
        bufferedWriter.close()
        Log.d("GENESIS", "transactions written")
    }
    return file
}
