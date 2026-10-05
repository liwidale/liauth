package com.liwidale.liauth.autofill

import android.app.PendingIntent
import android.app.assist.AssistStructure
import android.content.Intent
import android.os.CancellationSignal
import android.service.autofill.AutofillService
import android.service.autofill.Dataset
import android.service.autofill.FillCallback
import android.service.autofill.FillContext
import android.service.autofill.FillRequest
import android.service.autofill.FillResponse
import android.service.autofill.SaveCallback
import android.service.autofill.SaveRequest
import android.view.autofill.AutofillId
import android.view.autofill.AutofillValue
import android.widget.RemoteViews
import com.liwidale.liauth.MainActivity
import com.liwidale.liauth.R
import com.liwidale.liauth.vault.EngineHolder

/**
 * System autofill service that fills one-time codes. When an app or browser
 * shows a field that looks like an OTP input, the current codes from the
 * unlocked vault are offered; a locked vault offers to open LiAuth first.
 */
class LiAuthAutofillService : AutofillService() {

    override fun onFillRequest(
        request: FillRequest,
        cancellationSignal: CancellationSignal,
        callback: FillCallback,
    ) {
        val context: List<FillContext> = request.fillContexts
        val structure = context.lastOrNull()?.structure
        if (structure == null) {
            callback.onSuccess(null)
            return
        }

        val fields = collectOtpFields(structure)
        if (fields.isEmpty()) {
            callback.onSuccess(null)
            return
        }

        val engine = EngineHolder.get(this)
        if (!engine.isUnlocked()) {
            callback.onSuccess(lockedResponse(fields))
            return
        }

        val hint = structure.activityComponent?.packageName.orEmpty()
        val accounts = runCatching { engine.accounts() }.getOrDefault(emptyList())
        val codes = runCatching { engine.codes() }.getOrDefault(emptyList()).associateBy { it.id }
        if (accounts.isEmpty()) {
            callback.onSuccess(null)
            return
        }

        // Accounts whose issuer appears in the requesting package name go
        // first; everything else stays available below them.
        val ranked = accounts.sortedByDescending { account ->
            val issuer = account.issuer.lowercase().filter { it.isLetterOrDigit() }
            issuer.isNotEmpty() && hint.lowercase().contains(issuer)
        }

        val response = FillResponse.Builder()
        for (account in ranked.take(6)) {
            val code = codes[account.id]?.code ?: continue
            val label = buildString {
                append(account.issuer.ifEmpty { account.name })
                append(" · ")
                append(code)
            }
            val presentation = RemoteViews(packageName, R.layout.autofill_item).apply {
                setTextViewText(R.id.autofill_text, label)
            }
            val dataset = Dataset.Builder()
            for (field in fields) {
                dataset.setValue(field, AutofillValue.forText(code), presentation)
            }
            response.addDataset(dataset.build())
        }
        callback.onSuccess(response.build())
    }

    override fun onSaveRequest(request: SaveRequest, callback: SaveCallback) {
        // Codes are generated, never stored from forms.
        callback.onSuccess()
    }

    /** A single "Unlock LiAuth" entry that opens the app. */
    private fun lockedResponse(fields: List<AutofillId>): FillResponse {
        val intent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK
        }
        val pending = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE,
        )
        val presentation = RemoteViews(packageName, R.layout.autofill_item).apply {
            setTextViewText(R.id.autofill_text, getString(R.string.autofill_unlock))
        }
        return FillResponse.Builder()
            .setAuthentication(fields.toTypedArray(), pending.intentSender, presentation)
            .build()
    }

    private fun collectOtpFields(structure: AssistStructure): List<AutofillId> {
        val found = mutableListOf<AutofillId>()
        for (i in 0 until structure.windowNodeCount) {
            collect(structure.getWindowNodeAt(i).rootViewNode, found)
        }
        return found
    }

    private fun collect(node: AssistStructure.ViewNode, found: MutableList<AutofillId>) {
        val id = node.autofillId
        if (id != null && looksLikeOtpField(node)) {
            found.add(id)
        }
        for (i in 0 until node.childCount) {
            collect(node.getChildAt(i), found)
        }
    }

    private fun looksLikeOtpField(node: AssistStructure.ViewNode): Boolean {
        val hints = node.autofillHints.orEmpty().map { it.lowercase() }
        if (hints.any { "otp" in it || "sms" in it || "2fa" in it || "one-time" in it || "onetime" in it }) {
            return true
        }
        val haystack = listOfNotNull(node.idEntry, node.hint, node.text?.toString())
            .joinToString(" ")
            .lowercase()
        if (haystack.isEmpty()) return false
        return listOf("otp", "one-time", "one time", "2fa", "mfa", "verification code", "totp", "authenticator")
            .any { it in haystack }
    }
}
