package com.liwidale.liauth.core

import android.graphics.Bitmap
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.ImageProxy
import com.google.zxing.BarcodeFormat
import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.MultiFormatReader
import com.google.zxing.PlanarYUVLuminanceSource
import com.google.zxing.common.HybridBinarizer
import com.google.zxing.qrcode.QRCodeWriter

class QrAnalyzer(private val onResult: (String) -> Unit) : ImageAnalysis.Analyzer {

    private val reader = MultiFormatReader().apply {
        setHints(mapOf(DecodeHintType.POSSIBLE_FORMATS to listOf(BarcodeFormat.QR_CODE)))
    }

    override fun analyze(image: ImageProxy) {
        image.use { proxy ->
            runCatching {
                val plane = proxy.planes.firstOrNull() ?: return@runCatching null
                val buffer = plane.buffer
                buffer.rewind()
                val width = proxy.width
                val height = proxy.height
                val rowStride = plane.rowStride
                val pixelStride = plane.pixelStride
                val data = ByteArray(width * height)
                if (pixelStride == 1 && rowStride == width) {
                    buffer.get(data, 0, minOf(data.size, buffer.remaining()))
                } else {
                    val row = ByteArray(rowStride)
                    var offset = 0
                    for (y in 0 until height) {
                        buffer.position(y * rowStride)
                        val length = minOf(rowStride, buffer.remaining())
                        if (length <= 0) break
                        buffer.get(row, 0, length)
                        var x = 0
                        while (x < width && x * pixelStride < length) {
                            data[offset++] = row[x * pixelStride]
                            x++
                        }
                    }
                }
                val source = PlanarYUVLuminanceSource(data, width, height, 0, 0, width, height, false)
                reader.decodeWithState(BinaryBitmap(HybridBinarizer(source)))
            }.getOrNull()?.text?.takeIf { it.isNotEmpty() }?.let(onResult)
        }
    }
}

object QrGenerator {

    fun render(content: String, size: Int, dark: Boolean): Bitmap {
        val matrix = QRCodeWriter().encode(content, BarcodeFormat.QR_CODE, size, size)
        val foreground = if (dark) 0xFFF5F5F7.toInt() else 0xFF121214.toInt()
        val background = if (dark) 0xFF121214.toInt() else 0xFFFFFFFF.toInt()
        val pixels = IntArray(size * size) { index ->
            val x = index % size
            val y = index / size
            if (matrix[x, y]) foreground else background
        }
        return Bitmap.createBitmap(pixels, size, size, Bitmap.Config.ARGB_8888)
    }
}
