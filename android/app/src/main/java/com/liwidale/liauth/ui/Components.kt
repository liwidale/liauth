package com.liwidale.liauth.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.liwidale.liauth.ui.theme.ControlShape
import com.liwidale.liauth.ui.theme.LocalPalette

@Composable
fun PrimaryButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
) {
    val palette = LocalPalette.current
    Button(
        onClick = onClick,
        enabled = enabled,
        shape = ControlShape,
        colors = ButtonDefaults.buttonColors(
            containerColor = palette.accent,
            contentColor = palette.accentText,
            disabledContainerColor = palette.surfaceRaised,
            disabledContentColor = palette.textTertiary,
        ),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 0.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(40.dp),
    ) {
        Text(text, style = MaterialTheme.typography.labelLarge)
    }
}

@Composable
fun SecondaryButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val palette = LocalPalette.current
    Button(
        onClick = onClick,
        shape = ControlShape,
        colors = ButtonDefaults.buttonColors(
            containerColor = palette.surfaceRaised,
            contentColor = palette.textPrimary,
        ),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 0.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(40.dp)
            .border(1.dp, palette.border, ControlShape),
    ) {
        Text(text, style = MaterialTheme.typography.labelLarge)
    }
}

@Composable
fun LiAuthTextField(
    value: String,
    onValueChange: (String) -> Unit,
    hint: String,
    modifier: Modifier = Modifier,
    password: Boolean = false,
) {
    val palette = LocalPalette.current
    var revealed by remember { mutableStateOf(false) }
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        placeholder = { Text(hint, color = palette.textTertiary) },
        singleLine = true,
        visualTransformation = if (password && !revealed) {
            PasswordVisualTransformation()
        } else {
            VisualTransformation.None
        },
        trailingIcon = if (password) {
            {
                androidx.compose.material3.IconButton(onClick = { revealed = !revealed }) {
                    androidx.compose.material3.Icon(
                        if (revealed) LiAuthIcons.EyeOff else LiAuthIcons.Eye,
                        contentDescription = null,
                        tint = palette.textSecondary,
                    )
                }
            }
        } else {
            null
        },
        shape = ControlShape,
        colors = OutlinedTextFieldDefaults.colors(
            focusedContainerColor = palette.surfaceRaised,
            unfocusedContainerColor = palette.surfaceRaised,
            focusedBorderColor = palette.borderStrong,
            unfocusedBorderColor = palette.border,
            focusedTextColor = palette.textPrimary,
            unfocusedTextColor = palette.textPrimary,
            cursorColor = palette.textPrimary,
        ),
        modifier = modifier.fillMaxWidth(),
    )
}

@Composable
fun LiAuthSwitch(checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    val palette = LocalPalette.current
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        colors = SwitchDefaults.colors(
            checkedThumbColor = palette.accentText,
            checkedTrackColor = palette.accent,
            uncheckedThumbColor = palette.textSecondary,
            uncheckedTrackColor = palette.surfaceRaised,
            uncheckedBorderColor = palette.borderStrong,
        ),
    )
}

@Composable
fun CountdownBar(fraction: Float, modifier: Modifier = Modifier) {
    val palette = LocalPalette.current
    Canvas(
        modifier = modifier
            .fillMaxWidth()
            .height(2.dp),
    ) {
        drawRect(color = palette.border)
        drawRect(
            color = palette.textPrimary,
            size = Size(size.width * fraction.coerceIn(0f, 1f), size.height),
        )
    }
}

/** Primary brand colors for well-known services, keyed by normalized issuer. */
object BrandColors {
    private val colors: Map<String, androidx.compose.ui.graphics.Color> = buildMap {
        fun put(name: String, rgb: Long) = put(name, androidx.compose.ui.graphics.Color(0xFF000000 or rgb))
        put("github", 0x181717); put("gitlab", 0xFC6D26); put("google", 0x4285F4)
        put("gmail", 0x4285F4); put("youtube", 0xFF0000); put("microsoft", 0x0078D4)
        put("outlook", 0x0078D4); put("azure", 0x0078D4); put("apple", 0x0A84FF)
        put("icloud", 0x0A84FF); put("amazon", 0xFF9900); put("aws", 0xFF9900)
        put("facebook", 0x1877F2); put("meta", 0x1877F2); put("instagram", 0xE1306C)
        put("x", 0x1D9BF0); put("twitter", 0x1D9BF0); put("discord", 0x5865F2)
        put("slack", 0x4A154B); put("dropbox", 0x0061FF); put("steam", 0x1B2838)
        put("epicgames", 0x2C2C2C); put("riotgames", 0xEB0014); put("twitch", 0x9146FF)
        put("reddit", 0xFF4500); put("linkedin", 0x0A66C2); put("paypal", 0x003087)
        put("stripe", 0x635BFF); put("coinbase", 0x0052FF); put("binance", 0xF0B90B)
        put("kraken", 0x5729CE); put("protonmail", 0x6D4AFF); put("proton", 0x6D4AFF)
        put("bitwarden", 0x175DDC); put("nextcloud", 0x0082C9); put("cloudflare", 0xF38020)
        put("digitalocean", 0x0069FF); put("npm", 0xCB3837); put("docker", 0x2496ED)
        put("atlassian", 0x0052CC); put("jira", 0x0052CC); put("bitbucket", 0x0052CC)
        put("notion", 0x000000); put("figma", 0xA259FF); put("telegram", 0x24A1DE)
        put("whatsapp", 0x25D366); put("signal", 0x3A76F0); put("wordpress", 0x21759B)
        put("shopify", 0x5FBE42); put("ebay", 0xE63238); put("netflix", 0xE50914)
        put("spotify", 0x1ED760); put("nintendo", 0xE60012); put("playstation", 0x00439C)
        put("xbox", 0x107C10); put("yandex", 0xFFCC00); put("vk", 0x0077FF)
        put("mailru", 0x005FF9); put("heroku", 0x430098); put("netlify", 0x00ADB5)
        put("vercel", 0x000000); put("lastpass", 0xD52B1E); put("1password", 0x0A84FF)
    }

    fun forIssuer(issuer: String): androidx.compose.ui.graphics.Color? {
        val normalized = issuer.lowercase().filter { it.isLetterOrDigit() }
        return colors[normalized]
    }

    /// Simple Icons slugs bundled under assets/icons (white 96x96 PNGs, CC0).
    private val iconSlugs = setOf(
        "github", "gitlab", "google", "gmail", "youtube", "apple", "icloud",
        "facebook", "meta", "instagram", "x", "discord", "dropbox", "steam",
        "epicgames", "riotgames", "twitch", "reddit", "paypal", "stripe",
        "coinbase", "binance", "protonmail", "proton", "bitwarden", "nextcloud",
        "cloudflare", "digitalocean", "npm", "docker", "atlassian", "jira",
        "bitbucket", "notion", "figma", "telegram", "whatsapp", "signal",
        "wordpress", "shopify", "ebay", "netflix", "spotify", "playstation",
        "vk", "maildotru", "netlify", "vercel", "lastpass", "1password",
    )

    fun iconSlug(issuer: String): String? {
        val normalized = issuer.lowercase().filter { it.isLetterOrDigit() }
        val slug = when (normalized) {
            "twitter" -> "x"
            "mailru", "mail" -> "maildotru"
            else -> normalized
        }
        return slug.takeIf { it in iconSlugs }
    }
}

@Composable
fun Avatar(title: String, size: Dp, modifier: Modifier = Modifier, branded: Boolean = false) {
    val palette = LocalPalette.current
    val brand = if (branded) BrandColors.forIssuer(title) else null
    val slug = if (branded) BrandColors.iconSlug(title) else null

    if (slug != null) {
        // Real Simple Icons logo (white glyph) on the brand background.
        val context = androidx.compose.ui.platform.LocalContext.current
        val bitmap = remember(slug) {
            runCatching {
                context.assets.open("icons/$slug.png").use {
                    android.graphics.BitmapFactory.decodeStream(it)
                }
            }.getOrNull()
        }
        if (bitmap != null) {
            val background = brand ?: androidx.compose.ui.graphics.Color(0xFF1C1C1C)
            Box(
                contentAlignment = Alignment.Center,
                modifier = modifier
                    .size(size)
                    .background(background, ControlShape),
            ) {
                androidx.compose.foundation.Image(
                    bitmap = bitmap.asImageBitmap(),
                    contentDescription = title,
                    modifier = Modifier.size(size * 0.6f),
                )
            }
            return
        }
    }

    val background = brand ?: palette.surfaceRaised
    val borderColor = brand ?: palette.borderStrong
    val textColor = when {
        brand == null -> palette.textPrimary
        brand.luminance() > 0.6f -> androidx.compose.ui.graphics.Color.Black
        else -> androidx.compose.ui.graphics.Color.White
    }
    Box(
        contentAlignment = Alignment.Center,
        modifier = modifier
            .size(size)
            .background(background, ControlShape)
            .border(1.dp, borderColor, ControlShape),
    ) {
        Text(
            text = title.firstOrNull()?.uppercase() ?: "",
            style = MaterialTheme.typography.titleLarge,
            color = textColor,
            textAlign = TextAlign.Center,
        )
    }
}

/**
 * Text with the characters at [indices] (char positions from the fuzzy
 * matcher) rendered inverted, used to highlight search hits.
 */
@Composable
fun HighlightedText(
    text: String,
    indices: List<UInt>,
    style: androidx.compose.ui.text.TextStyle,
    color: androidx.compose.ui.graphics.Color,
) {
    val palette = LocalPalette.current
    if (indices.isEmpty()) {
        Text(text, style = style, color = color)
        return
    }
    val matched = indices.map { it.toInt() }.toSet()
    val annotated = androidx.compose.ui.text.buildAnnotatedString {
        text.forEachIndexed { i, ch ->
            if (i in matched) {
                withStyle(
                    androidx.compose.ui.text.SpanStyle(
                        background = palette.accent,
                        color = palette.accentText,
                    ),
                ) { append(ch) }
            } else {
                append(ch)
            }
        }
    }
    Text(annotated, style = style, color = color)
}

@Composable
fun PinMarker(modifier: Modifier = Modifier) {
    val palette = LocalPalette.current
    Box(
        modifier = modifier
            .size(6.dp)
            .background(palette.textSecondary, androidx.compose.foundation.shape.CircleShape),
    )
}

@Composable
fun SectionLabel(text: String, modifier: Modifier = Modifier) {
    val palette = LocalPalette.current
    Text(
        text = text.uppercase(),
        style = MaterialTheme.typography.labelSmall,
        color = palette.textTertiary,
        modifier = modifier.padding(bottom = 8.dp),
    )
}

fun formatCode(code: String): String = when (code.length) {
    6 -> "${code.substring(0, 3)} ${code.substring(3)}"
    8 -> "${code.substring(0, 4)} ${code.substring(4)}"
    else -> code
}
