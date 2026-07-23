package com.liwidale.liauth.ui

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

object LiAuthIcons {

    private fun icon(name: String, block: androidx.compose.ui.graphics.vector.ImageVector.Builder.() -> Unit): ImageVector =
        ImageVector.Builder(
            name = name,
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply(block).build()

    private fun androidx.compose.ui.graphics.vector.ImageVector.Builder.stroke(
        pathData: androidx.compose.ui.graphics.vector.PathBuilder.() -> Unit,
    ) {
        path(
            fill = null,
            stroke = SolidColor(Color.Black),
            strokeLineWidth = 1.8f,
            strokeLineCap = StrokeCap.Square,
            strokeLineJoin = StrokeJoin.Miter,
            pathFillType = PathFillType.NonZero,
            pathBuilder = pathData,
        )
    }

    private fun androidx.compose.ui.graphics.vector.ImageVector.Builder.solid(
        pathData: androidx.compose.ui.graphics.vector.PathBuilder.() -> Unit,
    ) {
        path(
            fill = SolidColor(Color.Black),
            stroke = null,
            pathFillType = PathFillType.NonZero,
            pathBuilder = pathData,
        )
    }

    /** Axis-aligned filled rectangle, the building block of the header icons. */
    private fun androidx.compose.ui.graphics.vector.ImageVector.Builder.bar(
        left: Float,
        top: Float,
        right: Float,
        bottom: Float,
    ) = solid {
        moveTo(left, top); lineTo(right, top); lineTo(right, bottom); lineTo(left, bottom); close()
    }

    // The header set mirrors the filled geometry drawn by the Windows client
    // (windows/src/views/widgets.rs) so both platforms read identically.
    private const val EDGE_LOW = 5.8f
    private const val EDGE_HIGH = 18.2f
    private const val SPAN = EDGE_HIGH - EDGE_LOW

    val Plus: ImageVector by lazy {
        icon("Plus") {
            val half = SPAN * 0.09f
            bar(12f - half, EDGE_LOW, 12f + half, EDGE_HIGH)
            bar(EDGE_LOW, 12f - half, EDGE_HIGH, 12f + half)
        }
    }

    val Sync: ImageVector by lazy {
        icon("Sync") {
            val half = SPAN * 0.07f
            val head = SPAN * 0.32f
            val top = EDGE_LOW + SPAN * 0.26f
            val bottom = EDGE_HIGH - SPAN * 0.26f
            bar(EDGE_LOW, top - half, EDGE_HIGH - head, top + half)
            solid {
                moveTo(EDGE_HIGH, top)
                lineTo(EDGE_HIGH - head, top - head * 0.7f)
                lineTo(EDGE_HIGH - head, top + head * 0.7f)
                close()
            }
            bar(EDGE_LOW + head, bottom - half, EDGE_HIGH, bottom + half)
            solid {
                moveTo(EDGE_LOW, bottom)
                lineTo(EDGE_LOW + head, bottom - head * 0.7f)
                lineTo(EDGE_LOW + head, bottom + head * 0.7f)
                close()
            }
        }
    }

    val Settings: ImageVector by lazy {
        icon("Settings") {
            val half = SPAN * 0.06f
            val knobRadius = SPAN * 0.13f
            listOf(0.18f to 0.72f, 0.5f to 0.28f, 0.82f to 0.55f).forEach { (row, knob) ->
                val y = EDGE_LOW + SPAN * row
                bar(EDGE_LOW, y - half, EDGE_HIGH, y + half)
                val x = EDGE_LOW + SPAN * knob
                solid {
                    moveTo(x - knobRadius, y)
                    arcToRelative(knobRadius, knobRadius, 0f, true, true, knobRadius * 2f, 0f)
                    arcToRelative(knobRadius, knobRadius, 0f, true, true, -knobRadius * 2f, 0f)
                    close()
                }
            }
        }
    }

    val Trash: ImageVector by lazy {
        icon("Trash") {
            val half = SPAN * 0.06f
            val lid = EDGE_LOW + SPAN * 0.18f
            bar(EDGE_LOW, lid - half, EDGE_HIGH, lid + half)
            bar(12f - SPAN * 0.18f, EDGE_LOW, 12f + SPAN * 0.18f, lid)
            solid {
                moveTo(EDGE_LOW + SPAN * 0.1f, lid + half * 2f)
                lineTo(EDGE_HIGH - SPAN * 0.1f, lid + half * 2f)
                lineTo(EDGE_HIGH - SPAN * 0.2f, EDGE_HIGH)
                lineTo(EDGE_LOW + SPAN * 0.2f, EDGE_HIGH)
                close()
            }
        }
    }

    val ChevronDown: ImageVector by lazy {
        icon("ChevronDown") {
            stroke { moveTo(6f, 9f); lineTo(12f, 15f); lineTo(18f, 9f) }
        }
    }

    val Back: ImageVector by lazy {
        icon("Back") {
            stroke { moveTo(19f, 12f); lineTo(5f, 12f) }
            stroke { moveTo(11f, 6f); lineTo(5f, 12f); lineTo(11f, 18f) }
        }
    }

    val Refresh: ImageVector by lazy {
        icon("Refresh") {
            stroke {
                moveTo(20f, 12f)
                arcTo(8f, 8f, 0f, isMoreThanHalf = true, isPositiveArc = false, 14.5f, 4.4f)
            }
            stroke { moveTo(20f, 5f); lineTo(20f, 12f); lineTo(13f, 12f) }
        }
    }

    val Close: ImageVector by lazy {
        icon("Close") {
            stroke { moveTo(6f, 6f); lineTo(18f, 18f) }
            stroke { moveTo(18f, 6f); lineTo(6f, 18f) }
        }
    }

    val Check: ImageVector by lazy {
        icon("Check") {
            stroke { moveTo(4f, 13f); lineTo(9f, 18f); lineTo(20f, 6f) }
        }
    }

    val Square: ImageVector by lazy {
        icon("Square") {
            stroke { moveTo(5f, 5f); lineTo(19f, 5f); lineTo(19f, 19f); lineTo(5f, 19f); close() }
        }
    }

    val Eye: ImageVector by lazy {
        icon("Eye") {
            stroke {
                moveTo(2f, 12f)
                curveTo(4.5f, 6.5f, 8f, 5f, 12f, 5f)
                curveTo(16f, 5f, 19.5f, 6.5f, 22f, 12f)
                curveTo(19.5f, 17.5f, 16f, 19f, 12f, 19f)
                curveTo(8f, 19f, 4.5f, 17.5f, 2f, 12f)
                close()
            }
            stroke {
                moveTo(15f, 12f)
                arcTo(3f, 3f, 0f, isMoreThanHalf = true, isPositiveArc = true, 9f, 12f)
                arcTo(3f, 3f, 0f, isMoreThanHalf = true, isPositiveArc = true, 15f, 12f)
                close()
            }
        }
    }

    val EyeOff: ImageVector by lazy {
        icon("EyeOff") {
            stroke {
                moveTo(4f, 12f)
                curveTo(6f, 8f, 9f, 6f, 12f, 6f)
                curveTo(13.2f, 6f, 14.4f, 6.3f, 15.5f, 6.9f)
            }
            stroke {
                moveTo(20f, 12f)
                curveTo(18.6f, 14.8f, 16.6f, 16.7f, 14.3f, 17.5f)
            }
            stroke {
                moveTo(9.5f, 9.5f)
                arcTo(3f, 3f, 0f, isMoreThanHalf = false, isPositiveArc = false, 14.5f, 14.5f)
            }
            stroke { moveTo(4f, 4f); lineTo(20f, 20f) }
        }
    }
}
