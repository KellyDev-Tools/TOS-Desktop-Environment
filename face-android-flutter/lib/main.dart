import 'package:flutter/material.dart';
import 'dart:ui';
import 'src/rust/api.dart';
import 'src/rust/frb_generated.dart';
import 'src/theme.dart';

Future<void> main() async {
  // Initialize Rust library
  await RustLib.init();
  runApp(const TOSFaceApp());
}

class TOSFaceApp extends StatelessWidget {
  const TOSFaceApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'TOS Face',
      theme: TOSTheme.darkTheme,
      home: const TOSHomeScreen(),
    );
  }
}

class TOSHomeScreen extends StatefulWidget {
  const TOSHomeScreen({super.key});

  @override
  State<TOSHomeScreen> createState() => _TOSHomeScreenState();
}

class _TOSHomeScreenState extends State<TOSHomeScreen> with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _fadeAnimation;
  String _status = "INITIALIZING CORE...";
  TosState? _tosState;
  List<String> _logs = [];
  bool _isBezelExpanded = false;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(seconds: 2),
      vsync: this,
    )..forward();

    _fadeAnimation = CurvedAnimation(
      parent: _controller,
      curve: Curves.easeIn,
    );
    
    _initTOS();
  }

  Future<void> _initTOS() async {
    try {
      final status = await getTosStatus();
      final initialState = await getInitialState();
      final logs = await getSystemLogs(state: initialState);
      
      setState(() {
        _status = status;
        _tosState = initialState;
        _logs = logs;
      });

      // Periodic log refresh simulation or actual poll
      _pollLogs();
    } catch (e) {
      setState(() {
        _status = "ERROR: LINK FAILURE\n$e";
      });
    }
  }

  void _pollLogs() async {
    while (mounted) {
      await Future.delayed(const Duration(seconds: 3));
      if (_tosState != null) {
        final logs = await getSystemLogs(state: _tosState!);
        if (logs.length != _logs.length) {
          setState(() {
            _logs = logs;
          });
        }
      }
    }
  }

  void _onExpandBezel() {
    setState(() {
      _isBezelExpanded = !_isBezelExpanded;
    });
  }

  Future<void> _changeLevel(int level) async {
    if (_tosState != null) {
      await setHierarchyLevel(state: _tosState!, level: level);
      await addLogEntry(state: _tosState!, text: "UI_ACTION: Hierarchy shifted to Level $level");
      final logs = await getSystemLogs(state: _tosState!);
      setState(() {
        _logs = logs;
      });
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          // Background Gradient
          Positioned.fill(
            child: Container(
              decoration: const BoxDecoration(
                gradient: RadialGradient(
                  center: Alignment(0, -0.5),
                  radius: 1.5,
                  colors: [
                    Color(0xFF1A1A2E),
                    TOSTheme.background,
                  ],
                ),
              ),
            ),
          ),
          
          // Abstract Glows
          Positioned(
            top: -100,
            left: -100,
            child: Container(
              width: 400,
              height: 400,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: TOSTheme.primary.withOpacity(0.05),
              ),
              child: BackdropFilter(
                filter: ImageFilter.blur(sigmaX: 100, sigmaY: 100),
                child: Container(),
              ),
            ),
          ),
          
          // Main UI
          SafeArea(
            child: Center(
              child: FadeTransition(
                opacity: _fadeAnimation,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    // TOS Logo Concept with pulse effect
                    _buildAnimatedLogo(),
                    const SizedBox(height: 40),
                    
                    // Glassmorphic Status Box
                    _buildGlassBox(
                      title: "CORE STATUS",
                      content: _status,
                      width: 340,
                    ),
                    
                    const SizedBox(height: 20),
                    
                    // System Logs Box
                    _buildGlassBox(
                      title: "SYSTEM LOG",
                      content: _logs.isEmpty ? "NO LOG ENTRIES" : _logs.reversed.take(5).join("\n"),
                      width: 340,
                      height: 140,
                      isMonospace: true,
                    ),
                    
                    const SizedBox(height: 40),
                    
                    // Level Control Bar
                    _buildLevelBar(),
                  ],
                ),
              ),
            ),
          ),

          // Tactical Side Bezel
          _buildTacticalBezel(),

          // Bezel Control Button
          _buildBezelToggle(),
        ],
      ),
    );
  }

  Widget _buildAnimatedLogo() {
    return Container(
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        boxShadow: [
          BoxShadow(
            color: TOSTheme.primary.withOpacity(0.2),
            blurRadius: 20,
            spreadRadius: 5,
          ),
        ],
        gradient: const LinearGradient(
          colors: [TOSTheme.primary, TOSTheme.warning],
        ),
      ),
      child: CircleAvatar(
        radius: 45,
        backgroundColor: TOSTheme.background,
        child: const Text(
          "T",
          style: TextStyle(
            fontSize: 48,
            fontFamily: 'Outfit',
            fontWeight: FontWeight.w900,
            color: TOSTheme.primary,
            letterSpacing: -2,
          ),
        ),
      ),
    );
  }

  Widget _buildGlassBox({
    required String title,
    required String content,
    double width = 320,
    double? height,
    bool isMonospace = false,
  }) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(TOSTheme.radiusLg),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: TOSTheme.glassBlur, sigmaY: TOSTheme.glassBlur),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 500),
          width: width,
          height: height,
          padding: const EdgeInsets.all(20),
          decoration: BoxDecoration(
            color: TOSTheme.glassBg,
            borderRadius: BorderRadius.circular(TOSTheme.radiusLg),
            border: Border.all(
              color: TOSTheme.glassBorder,
              width: 1,
            ),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withOpacity(0.3),
                blurRadius: 10,
                offset: const Offset(0, 4),
              ),
            ],
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                children: [
                  Container(
                    width: 4,
                    height: 12,
                    color: TOSTheme.primary,
                  ),
                  const SizedBox(width: 8),
                  Text(
                    title,
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.w900,
                      color: TOSTheme.primary.withOpacity(0.7),
                      letterSpacing: 2,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              Text(
                content,
                style: TextStyle(
                  fontSize: 13,
                  height: 1.5,
                  fontFamily: isMonospace ? 'Courier' : null,
                  color: TOSTheme.text.withOpacity(0.9),
                ),
                maxLines: height != null ? 5 : null,
                overflow: height != null ? TextOverflow.ellipsis : null,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildLevelBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: TOSTheme.surface.withOpacity(0.5),
        borderRadius: BorderRadius.circular(TOSTheme.radiusPill),
        border: Border.all(color: TOSTheme.border),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [1, 2, 3, 4].map((l) {
          return Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4),
            child: InkWell(
              onTap: () => _changeLevel(l),
              borderRadius: BorderRadius.circular(20),
              child: Container(
                width: 40,
                height: 40,
                alignment: Alignment.center,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: Colors.transparent,
                ),
                child: Text(
                  "L$l",
                  style: const TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                    color: TOSTheme.textDim,
                  ),
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }

  Widget _buildTacticalBezel() {
    return AnimatedPositioned(
      duration: const Duration(milliseconds: 400),
      curve: Curves.easeOutCubic,
      right: _isBezelExpanded ? 0 : -300,
      top: 60,
      bottom: 60,
      child: GestureDetector(
        onHorizontalDragEnd: (details) {
          if (details.primaryVelocity! > 0) {
            setState(() => _isBezelExpanded = false);
          }
        },
        child: ClipRRect(
          borderRadius: const BorderRadius.horizontal(left: Radius.circular(TOSTheme.radiusElbow)),
          child: BackdropFilter(
            filter: ImageFilter.blur(sigmaX: 25, sigmaY: 25),
            child: Container(
              width: 320,
              decoration: BoxDecoration(
                color: TOSTheme.surfaceOverlay,
                borderRadius: const BorderRadius.horizontal(left: Radius.circular(TOSTheme.radiusElbow)),
                border: Border.all(
                  color: TOSTheme.primary.withOpacity(0.2),
                  width: 1.5,
                ),
              ),
              child: Column(
                children: [
                  const SizedBox(height: 40),
                  const Text(
                    "TACTICAL OVERVIEW",
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 4,
                      color: TOSTheme.primary,
                    ),
                  ),
                  const SizedBox(height: 30),
                  Expanded(
                    child: ListView(
                      padding: const EdgeInsets.symmetric(horizontal: 24),
                      children: [
                        _buildSectorTile("GLOBAL-ALPHA", true),
                        _buildSectorTile("LOCAL-BETA", false),
                        _buildSectorTile("SECURE-GATE", false),
                        const SizedBox(height: 20),
                        const Text(
                          "ACTIVE MODULES",
                          style: TextStyle(fontSize: 10, color: TOSTheme.textMuted, letterSpacing: 2),
                        ),
                        const SizedBox(height: 10),
                        _buildModuleItem("AI-CORE", "ACTIVE"),
                        _buildModuleItem("NET-STACK", "IDLE"),
                        _buildModuleItem("SANDBOX", "LOCKED"),
                      ],
                    ),
                  ),
                  _buildBezelFooter(),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBezelToggle() {
    return Positioned(
      right: 20,
      top: 20,
      child: InkWell(
        onTap: _onExpandBezel,
        borderRadius: BorderRadius.circular(25),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 300),
          width: 50,
          height: 50,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: _isBezelExpanded ? TOSTheme.primary : TOSTheme.surface.withOpacity(0.5),
            boxShadow: [
              if (_isBezelExpanded)
                BoxShadow(color: TOSTheme.primary.withOpacity(0.3), blurRadius: 15),
            ],
          ),
          child: Icon(
            _isBezelExpanded ? Icons.close : Icons.radar,
            color: _isBezelExpanded ? Colors.black : TOSTheme.primary,
            size: 20,
          ),
        ),
      ),
    );
  }

  Widget _buildSectorTile(String name, bool active) {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: active ? TOSTheme.primary.withOpacity(0.05) : Colors.transparent,
        borderRadius: BorderRadius.circular(TOSTheme.radiusLg),
        border: Border.all(
          color: active ? TOSTheme.primary.withOpacity(0.3) : TOSTheme.border,
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              name,
              style: TextStyle(
                color: active ? TOSTheme.text : TOSTheme.textMuted,
                fontWeight: active ? FontWeight.bold : FontWeight.normal,
                fontSize: 14,
              ),
            ),
          ),
          if (active)
            const Icon(Icons.circle, color: TOSTheme.primary, size: 8),
        ],
      ),
    );
  }

  Widget _buildModuleItem(String name, String status) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Text(name, style: const TextStyle(color: TOSTheme.textDim, fontSize: 12)),
          const Spacer(),
          Text(
            status,
            style: TextStyle(
              color: status == "ACTIVE" ? TOSTheme.success : TOSTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBezelFooter() {
    return Container(
      padding: const EdgeInsets.all(24),
      child: Row(
        children: [
          const Icon(Icons.shield_outlined, color: TOSTheme.textMuted, size: 16),
          const SizedBox(width: 8),
          const Text("SECURE_LINK", style: TextStyle(color: TOSTheme.textMuted, fontSize: 10)),
          const Spacer(),
          Text("v1.0-BETA", style: TextStyle(color: TOSTheme.primary.withOpacity(0.5), fontSize: 10)),
        ],
      ),
    );
  }
}
