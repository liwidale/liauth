package com.liwidale.liauth.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.liwidale.liauth.R

/**
 * Vercel-style design tokens. Material 3 is only the mechanical base; every
 * color, shape and type style is overridden here.
 */
data class LiAuthPalette(
    val background: Color,
    val surface: Color,
    val surfaceRaised: Color,
    val glass: Color,
    val border: Color,
    val borderStrong: Color,
    val hover: Color,
    val textPrimary: Color,
    val textSecondary: Color,
    val textTertiary: Color,
    val accent: Color,
    val accentText: Color,
    val danger: Color,
    val success: Color,
    val warning: Color,
)

val DarkPalette = LiAuthPalette(
    background = Color(0xFF000000),
    surface = Color(0xFF090909),
    surfaceRaised = Color(0xFF111111),
    glass = Color(0xFF090909),
    border = Color(0xFF262626),
    borderStrong = Color(0xFF3F3F46),
    hover = Color(0xFF18181B),
    textPrimary = Color(0xFFFAFAFA),
    textSecondary = Color(0xFFA1A1AA),
    textTertiary = Color(0xFF71717A),
    accent = Color(0xFFFFFFFF),
    accentText = Color(0xFF000000),
    danger = Color(0xFFDC2626),
    success = Color(0xFF16A34A),
    warning = Color(0xFFF59E0B),
)

val LightPalette = LiAuthPalette(
    background = Color(0xFFFFFFFF),
    surface = Color(0xFFFFFFFF),
    surfaceRaised = Color(0xFFFAFAFA),
    glass = Color(0xFFFFFFFF),
    border = Color(0xFFEAEAEA),
    borderStrong = Color(0xFFD4D4D8),
    hover = Color(0xFFF4F4F5),
    textPrimary = Color(0xFF0A0A0A),
    textSecondary = Color(0xFF52525B),
    textTertiary = Color(0xFFA1A1AA),
    accent = Color(0xFF0A0A0A),
    accentText = Color(0xFFFFFFFF),
    danger = Color(0xFFDC2626),
    success = Color(0xFF16A34A),
    warning = Color(0xFFF59E0B),
)

val LocalPalette = staticCompositionLocalOf { DarkPalette }

/** Corner radii of the design system. */
val SmallShape = RoundedCornerShape(4.dp)
val ControlShape = RoundedCornerShape(6.dp)
val CardShape = RoundedCornerShape(8.dp)
val DialogShape = RoundedCornerShape(12.dp)

val Inter = FontFamily(
    Font(R.font.inter_regular, FontWeight.Normal),
    Font(R.font.inter_medium, FontWeight.Medium),
    Font(R.font.inter_semibold, FontWeight.SemiBold),
    Font(R.font.inter_bold, FontWeight.Bold),
)

val CodeFont = FontFamily(
    Font(R.font.jetbrainsmono_semibold, FontWeight.SemiBold),
)

val CodeTextStyle = TextStyle(
    fontFamily = CodeFont,
    fontWeight = FontWeight.SemiBold,
    fontSize = 24.sp,
)

/** Type scale: 12/13/14/16/20/24/32, weights capped at 700. */
private val LiAuthTypography = Typography(
    displayLarge = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Bold, fontSize = 32.sp),
    headlineMedium = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Bold, fontSize = 24.sp),
    titleLarge = TextStyle(fontFamily = Inter, fontWeight = FontWeight.SemiBold, fontSize = 20.sp),
    titleMedium = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Medium, fontSize = 14.sp),
    bodyLarge = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Normal, fontSize = 14.sp),
    bodyMedium = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Normal, fontSize = 13.sp),
    bodySmall = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Normal, fontSize = 12.sp),
    labelLarge = TextStyle(fontFamily = Inter, fontWeight = FontWeight.Medium, fontSize = 14.sp),
    labelSmall = TextStyle(
        fontFamily = Inter,
        fontWeight = FontWeight.Medium,
        fontSize = 12.sp,
        letterSpacing = 0.8.sp,
    ),
)

private val LiAuthShapes = Shapes(
    extraSmall = SmallShape,
    small = ControlShape,
    medium = CardShape,
    large = DialogShape,
    extraLarge = DialogShape,
)

@Composable
fun LiAuthTheme(themeMode: String, content: @Composable () -> Unit) {
    val dark = when (themeMode) {
        "light" -> false
        "dark" -> true
        else -> isSystemInDarkTheme()
    }
    val palette = if (dark) DarkPalette else LightPalette
    val colorScheme = if (dark) {
        darkColorScheme(
            primary = palette.accent,
            onPrimary = palette.accentText,
            background = palette.background,
            onBackground = palette.textPrimary,
            surface = palette.surface,
            onSurface = palette.textPrimary,
            surfaceVariant = palette.surfaceRaised,
            onSurfaceVariant = palette.textSecondary,
            outline = palette.border,
            error = palette.danger,
        )
    } else {
        lightColorScheme(
            primary = palette.accent,
            onPrimary = palette.accentText,
            background = palette.background,
            onBackground = palette.textPrimary,
            surface = palette.surface,
            onSurface = palette.textPrimary,
            surfaceVariant = palette.surfaceRaised,
            onSurfaceVariant = palette.textSecondary,
            outline = palette.border,
            error = palette.danger,
        )
    }

    androidx.compose.runtime.CompositionLocalProvider(LocalPalette provides palette) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = LiAuthTypography,
            shapes = LiAuthShapes,
            content = content,
        )
    }
}
