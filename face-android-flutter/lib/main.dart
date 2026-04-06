import 'package:flutter/material.dart';
import 'dart:ui';

void main() {
  runApp(const TOSFaceApp());
}

class TOSFaceApp extends StatelessWidget {
  const TOSFaceApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'TOS Face',
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: const Color(0xFF030508),
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF00D4FF),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
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
  String _status = "Initializing...";

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(seconds: 3),
      vsync: this,
    )..forward();

    _fadeAnimation = CurvedAnimation(
      parent: _controller,
      curve: Curves.easeIn,
    );
    
    // Simulated state transition
    Future.delayed(const Duration(seconds: 2), () {
      setState(() {
        _status = "TOS System: ONLINE\nProtocol: Flutter/Rust Bridge\nUI: Premium Glassmorphism\nActive Sector: Primary";
      });
    });
  }

  void _onExpandBezel() {
    setState(() {
      _isBezelExpanded = !_isBezelExpanded;
    });
  }

  bool _isBezelExpanded = false;

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
                    Color(0xFF00384D),
                    Color(0xFF030508),
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
              width: 300,
              height: 300,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: const Color(0xFF00D4FF).withOpacity(0.1),
              ),
              child: BackdropFilter(
                filter: ImageFilter.blur(sigmaX: 80, sigmaY: 80),
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
                    // TOS Logo Concept
                    Container(
                      padding: const EdgeInsets.all(2),
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        gradient: const LinearGradient(
                          colors: [Color(0xFF00D4FF), Color(0xFF00FFC2)],
                        ),
                      ),
                      child: CircleAvatar(
                        radius: 50,
                        backgroundColor: const Color(0xFF030508),
                        child: const Text(
                          "T",
                          style: TextStyle(
                            fontSize: 40,
                            fontWeight: FontWeight.w900,
                            color: Color(0xFF00D4FF),
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(height: 40),
                    
                    // Glassmorphic Status Box
                    ClipRRect(
                      borderRadius: BorderRadius.circular(20),
                      child: BackdropFilter(
                        filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
                        child: Container(
                          width: 320,
                          padding: const EdgeInsets.all(24),
                          decoration: BoxDecoration(
                            color: Colors.white.withOpacity(0.05),
                            borderRadius: BorderRadius.circular(20),
                            border: Border.all(
                              color: Colors.white.withOpacity(0.1),
                              width: 1,
                            ),
                          ),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                "BOOT SEQUENCE",
                                style: TextStyle(
                                  fontSize: 12,
                                  fontWeight: FontWeight.bold,
                                  color: const Color(0xFF00D4FF).withOpacity(0.8),
                                  letterSpacing: 2,
                                ),
                              ),
                              const SizedBox(height: 16),
                              Text(
                                _status,
                                style: const TextStyle(
                                  fontSize: 14,
                                  height: 1.6,
                                  fontFamily: 'Courier',
                                  color: Colors.white,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                    
                    const SizedBox(height: 60),
                    
                    // Loading indicator
                    const SizedBox(
                      width: 200,
                      child: LinearProgressIndicator(
                        backgroundColor: Colors.white10,
                        color: Color(0xFF00D4FF),
                        minHeight: 2,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
          // Tactical Side Bezel
          AnimatedPositioned(
            duration: const Duration(milliseconds: 300),
            curve: Curves.easeInOut,
            right: _isBezelExpanded ? 0 : -280,
            top: 100,
            bottom: 100,
            child: GestureDetector(
              onHorizontalDragEnd: (details) {
                if (details.primaryVelocity! < 0) {
                  setState(() => _isBezelExpanded = true);
                } else if (details.primaryVelocity! > 0) {
                  setState(() => _isBezelExpanded = false);
                }
              },
              child: ClipRRect(
                borderRadius: const BorderRadius.horizontal(left: Radius.circular(30)),
                child: BackdropFilter(
                  filter: ImageFilter.blur(sigmaX: 20, sigmaY: 20),
                  child: Container(
                    width: 300,
                    decoration: BoxDecoration(
                      color: const Color(0xFF001F29).withOpacity(0.8),
                      borderRadius: const BorderRadius.horizontal(left: Radius.circular(30)),
                      border: Border.all(
                        color: const Color(0xFF00D4FF).withOpacity(0.3),
                        width: 1.5,
                      ),
                    ),
                    child: Column(
                      children: [
                        const SizedBox(height: 30),
                        const Text(
                          "TACTICAL SECTORS",
                          style: TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.bold,
                            letterSpacing: 3,
                            color: Color(0xFF00D4FF),
                          ),
                        ),
                        const SizedBox(height: 20),
                        const Divider(color: Colors.white10),
                        Expanded(
                          child: ListView(
                            padding: const EdgeInsets.symmetric(horizontal: 20),
                            children: [
                              _buildSectorTile("Primary", true),
                              _buildSectorTile("Intel-Core", false),
                              _buildSectorTile("Network-Ops", false),
                            ],
                          ),
                        ),
                        // Bezel handle
                        Padding(
                          padding: const EdgeInsets.all(20),
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              IconButton(
                                icon: const Icon(Icons.settings_outlined, color: Colors.white54),
                                onPressed: () {},
                              ),
                              const Spacer(),
                              const Text("TOS v1.0", style: TextStyle(color: Colors.white24, fontSize: 10)),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),

          // Bezel Control Button
          Positioned(
            right: 20,
            top: 20,
            child: InkWell(
              onTap: _onExpandBezel,
              child: AnimatedRotation(
                turns: _isBezelExpanded ? 0.5 : 0,
                duration: const Duration(milliseconds: 300),
                child: Container(
                  width: 50,
                  height: 50,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: _isBezelExpanded ? const Color(0xFF00D4FF) : Colors.white10,
                    border: Border.all(color: const Color(0xFF00D4FF).withOpacity(0.5)),
                  ),
                  child: Icon(
                    _isBezelExpanded ? Icons.close : Icons.menu_open,
                    color: _isBezelExpanded ? Colors.black : const Color(0xFF00D4FF),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectorTile(String name, bool active) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: active ? const Color(0xFF00D4FF).withOpacity(0.1) : Colors.transparent,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: active ? const Color(0xFF00D4FF).withOpacity(0.4) : Colors.white10,
        ),
      ),
      child: ListTile(
        title: Text(
          name,
          style: TextStyle(
            color: active ? Colors.white : Colors.white54,
            fontWeight: active ? FontWeight.bold : FontWeight.normal,
          ),
        ),
        trailing: active 
          ? const Icon(Icons.radar, color: Color(0xFF00D4FF), size: 18)
          : null,
      ),
    );
  }
}
