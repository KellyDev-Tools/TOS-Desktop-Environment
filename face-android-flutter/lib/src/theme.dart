import 'package:flutter/material.dart';

/// TOS Design Tokens for Flutter
/// Based on assets/design_tokens.json
class TOSTheme {
  // Brand Colors (Amber Theme)
  static const Color primary = Color(0xFFF7A833);
  static const Color secondary = Color(0xFFCC99CC);
  static const Color accent = Color(0xFF5C88DA);
  
  // Status Colors
  static const Color success = Color(0xFF66CC66);
  static const Color warning = Color(0xFFFF7744);
  static const Color danger = Color(0xFFEE4444);
  
  // UI Backgrounds
  static const Color background = Color(0xFF0A0A14);
  static const Color surface = Color(0xFF12121F);
  static const Color surfaceRaised = Color(0xFF1A1A2E);
  static const Color surfaceOverlay = Color(0xEB12121F); // 0.92 alpha
  
  // Text Colors
  static const Color text = Color(0xFFE0E0E8);
  static const Color textDim = Color(0x8CE0E0E8); // 0.55 alpha
  static const Color textMuted = Color(0x59E0E0E8); // 0.35 alpha
  
  // Glassmorphism
  static const Color glassBg = Color(0xA60F0F19); // 0.65 alpha
  static const Color glassBorder = Color(0x0FFFFFFF); // 0.06 alpha
  static const double glassBlur = 16.0;
  
  // Borders
  static const Color border = Color(0x14FFFFFF); // 0.08 alpha
  static const Color borderActive = Color(0x66F7A833); // 0.4 alpha

  // Layout Constants
  static const double radiusSm = 4.0;
  static const double radiusMd = 8.0;
  static const double radiusLg = 16.0;
  static const double radiusPill = 32.0;
  static const double radiusElbow = 32.0;

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: background,
      colorScheme: const ColorScheme.dark(
        primary: primary,
        secondary: secondary,
        surface: surface,
        error: danger,
      ),
      useMaterial3: true,
      fontFamily: 'Inter',
      textTheme: const TextTheme(
        bodyLarge: TextStyle(color: text),
        bodyMedium: TextStyle(color: text),
        displayLarge: TextStyle(fontFamily: 'Outfit', fontWeight: FontWeight.bold),
      ),
    );
  }
}
