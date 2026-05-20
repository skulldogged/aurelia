package com.aurelia.app.ai

import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.File

data class OnDeviceAiModel(
  val id: String,
  val name: String,
  val fileName: String,
  val downloadUrl: String,
  val sizeLabel: String,
)

sealed interface AiModelDownloadState {
  data object Idle : AiModelDownloadState
  data class Ready(val path: String) : AiModelDownloadState
  data class Missing(val expectedPath: String?) : AiModelDownloadState
  data class Downloading(
    val modelName: String,
    val bytesRead: Long,
    val totalBytes: Long?,
  ) : AiModelDownloadState
  data class Error(val message: String) : AiModelDownloadState
}

object OnDeviceAiModels {
  val default = OnDeviceAiModel(
    id = "gemma-4-e2b-it",
    name = "Gemma 4 E2B IT",
    fileName = "gemma-4-E2B-it.litertlm",
    downloadUrl = "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/main/gemma-4-E2B-it.litertlm?download=true",
    sizeLabel = "2.6 GB",
  )
}

class AiModelDownloader(
  private val client: OkHttpClient = OkHttpClient(),
) {
  fun download(
    model: OnDeviceAiModel,
    modelsDir: File,
    onProgress: (bytesRead: Long, totalBytes: Long?) -> Unit,
  ): File {
    modelsDir.mkdirs()
    val destination = File(modelsDir, model.fileName)
    val partial = File(modelsDir, "${model.fileName}.download")
    val request = Request.Builder().url(model.downloadUrl).build()

    client.newCall(request).execute().use { response ->
      if (!response.isSuccessful) {
        error("Model download failed: HTTP ${response.code}")
      }

      val body = response.body
      val totalBytes = body.contentLength().takeIf { it > 0L }
      var bytesRead = 0L

      body.byteStream().use { input ->
        partial.outputStream().use { output ->
          val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
          while (true) {
            val read = input.read(buffer)
            if (read == -1) break
            output.write(buffer, 0, read)
            bytesRead += read
            onProgress(bytesRead, totalBytes)
          }
        }
      }

      if (destination.exists() && !destination.delete()) {
        error("Could not replace existing model file")
      }
      if (!partial.renameTo(destination)) {
        error("Could not finalize downloaded model")
      }
      return destination
    }
  }
}
