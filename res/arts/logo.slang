// logo.slang — Entelecheia Logo Shader
// Authentic Pencil Texture with True 3D Overlap

#define PI 3.14159265358979323846
#define TAU 6.28318530717958647692
#define CURVE_SAMPLES 800
#define STROKE_COUNT 100
#define RIBBON_W 0.075

RWTexture2D<float4> outputTexture : register(u0);
static const float _time = 0.0;
static const float _animSpeed = 1.0;

float hash(float n) { return frac(sin(n * 127.1) * 43758.5453); }
float hash2(float2 p) { return frac(sin(dot(p, float2(127.1, 311.7))) * 43758.5453); }
float vnoise(float2 p) {
    float2 i = floor(p), f = frac(p);
    f = f * f * (3.0 - 2.0 * f);
    return lerp(lerp(hash2(i), hash2(i + float2(1,0)), f.x),
                lerp(hash2(i + float2(0,1)), hash2(i + float2(1,1)), f.x), f.y);
}

// Generate the 3D curve path for the logo (infinity symbol shape)
float3 curve3D(float t) {
    float px = sin(t);
    // 1. Open up the loops (was 0.25, reduced to 0.18 for a sleeker, more elegant shape)
    float py = -sin(2.0 * t) * 0.18; 
    
    // 2. REMOVED Global Asymmetry Scaling. 
    // The "skeleton" (inner holes) is now geometrically perfectly symmetrical.
    // Asymmetry is now handled entirely by line looseness (ribbon_width) in renderLayer.
    
    // 3. Top-Left Stretch (非常轻微的拉扯，避免突兀)
    // Isolate the top-left quadrant smoothly (px < 0, py > 0)
    float topLeftWeight = smoothstep(0.1, -0.9, px) * smoothstep(-0.05, 0.25, py);
    px -= 0.04 * topLeftWeight; // Pull slightly further left
    py += 0.02 * topLeftWeight; // Pull slightly further up
    
    // Depth: t=0 (center, moving down-right) is Front (+1)
    // t=PI (center, moving down-left) is Back (-1)
    float pz = cos(t);
    
    // 3D Twist along X-axis
    // Rotate Y and Z around X
    float twist_angle = px * 0.15; // Slightly reduced twist
    float cy = cos(twist_angle);
    float sy = sin(twist_angle);
    
    float new_py = py * cy - pz * sy;
    float new_pz = py * sy + pz * cy;
    py = new_py;
    pz = new_pz;
    
    // Scale and return - slightly larger overall
    return float3(px * 0.75, py * 0.75, pz);
}

float2 curvePt(float t) { return curve3D(t).xy; }

float2 curveTan(float t) {
    float e = 0.001;
    return normalize(curvePt(t + e) - curvePt(t - e));
}
float2 curveNor(float t) {
    float2 tanDir = curveTan(t);
    return float2(-tanDir.y, tanDir.x);
}

float3 getColor(float t, float pos, float seed) {
    // Multiply by 2.0 so the 4-color cycle repeats twice over the entire curve
    float nt = frac(t * 2.0 / TAU);
    
    // Muted/Pastel colored pencil tones from the reference
    float3 yellow = float3(1.0, 0.9, 0.4);
    float3 purple = float3(0.6, 0.4, 0.9);
    float3 cyan   = float3(0.3, 0.6, 1.0);
    float3 pink   = float3(0.95, 0.5, 0.8);
    
    float3 c;
    if (nt < 0.25) {
        c = lerp(yellow, purple, smoothstep(0.05, 0.20, nt));
    } else if (nt < 0.5) {
        c = lerp(purple, cyan, smoothstep(0.30, 0.45, nt));
    } else if (nt < 0.75) {
        c = lerp(cyan, pink, smoothstep(0.55, 0.70, nt));
    } else {
        c = lerp(pink, yellow, smoothstep(0.80, 0.95, nt));
    }
    
    // Add per-stroke hue shift
    float hueShift = (hash(seed * 11.2) - 0.5) * 0.08;
    c += hueShift;
    
    // Luminance variance for pencil strokes (some light, some heavy)
    float brightness = 0.5 + 0.7 * hash(seed * 13.4);
    c *= brightness;
    
    // Inject some graphite (dark grey) and white highlights for extreme contrast/texture
    float special = hash(seed * 15.3);
    if (special > 0.92) {
        c = float3(1.0, 1.0, 1.0); // Bright white scratch
    } else if (special > 0.78) {
        c = float3(0.15, 0.15, 0.2); // Dark graphite stroke
    }
    
    return saturate(c);
}

// Render a single ribbon layer (front or back branch)
float4 renderLayer(float2 p, float base_t, float color_flow, float stroke_flow, float z_depth) {
    float2 cp = curvePt(base_t);
    float distToCurve = length(p - cp);
    
    // Asymmetry: The left side's lines are looser (wider), right side's lines are tighter.
    // cos(base_t) ranges from -1 (Left) to +1 (Right)
    float cx = cos(base_t);
    // Left side gets 1.2x width, Right side gets 0.8x width
    float looseness = 1.0 - 0.20 * cx; 
    
    // Tapering width: thicker in the center crossover, thinner at the far left/right edges
    float ribbon_width = RIBBON_W * looseness * (0.35 + 0.65 * pow(abs(cx), 2.0));
    
    // Early exit if pixel is nowhere near this branch
    if (distToCurve > ribbon_width * 1.5) return float4(0);
    
    float2 tanDir = curveTan(base_t);
    float2 norDir = curveNor(base_t);
    
    float3 layerRgb = float3(0);
    float layerAlpha = 0;
    
    [unroll]
    for(int i = 0; i < STROKE_COUNT; i++) {
        float seed = float(i) * 17.3 + 1.0;
        
        // Distribution of strokes
        float rawPos = hash(seed);
        float pos = (rawPos - 0.5) * 2.0; 
        
        // Pull strokes closer together towards the center of the ribbon (gathered look)
        pos = sign(pos) * pow(abs(pos), 1.3); 
        
        // Ribbon offset
        float offset = pos * ribbon_width;
        
        // Jitter and flow - MUST use integer frequency for seamless wrap at curve ends
        float freq = floor(2.0 + hash(seed) * 5.0);
        float wobble = sin(base_t * freq + seed * 10.0) * 0.005; 
        float2 strokePt = cp + norDir * (offset + wobble);
        float d = length(p - strokePt);
        
        // Finer pencil strokes for a less thick/bulky appearance
        float strokeW = 0.0007 + hash(seed+1) * 0.0014; 
        if (d > strokeW * 3.0) continue;
        
        // Crisp antialiased stroke line
        float alpha = exp(-d*d/(strokeW*strokeW * 1.5));
        
        // Staggered line start and ends (错位) - MUST use integer segments for seamless wrap
        float num_segments = floor(2.0 + hash(seed * 4.4) * 3.0); 
        float segment_phase = hash(seed * 5.5) * TAU;
        
        // Smooth long strokes
        float envelope = sin(base_t * num_segments + segment_phase);
        float macro_dash = smoothstep(-0.3, 0.3, envelope);
        alpha *= macro_dash;
        
        // Directional pencil scratching - Map base_t to a circle to make noise seamlessly periodic
        float2 seamless_uv = float2(cos(base_t - stroke_flow), sin(base_t - stroke_flow)) * 30.0;
        float noise = vnoise(seamless_uv + float2(pos * 50.0, seed * 11.0));
        float fine_noise = vnoise(p * 800.0);
        
        float breakPattern = smoothstep(0.2, 0.8, noise * 0.6 + fine_noise * 0.4 + 0.2);
        alpha *= breakPattern;
        
        // Pressure variation
        float pressure = 0.4 + 0.6 * hash(seed+3);
        alpha *= pressure;
        
        // Soft drop shadow / ambient occlusion based on Z depth
        // If this layer is in the back (z_depth < 0), darken it slightly
        float depth_shade = lerp(0.5, 1.0, z_depth * 0.5 + 0.5);
        
        // Pass pos to getColor to differentiate inner/outer edges
        float3 col = getColor(base_t - color_flow, pos, seed) * depth_shade;
        
        // Standard alpha accumulation (approximated for unordered strokes)
        float invA = 1.0 - alpha;
        layerRgb = col * alpha + layerRgb * invA;
        layerAlpha = alpha + layerAlpha * invA;
    }
    
    // To maintain strong vibrancy without blowing out, boost alpha slightly
    layerAlpha = saturate(layerAlpha * 1.2);
    
    // Un-premultiply layerRgb so it returns straight RGB
    if (layerAlpha > 0.001) {
        layerRgb /= layerAlpha;
    }
    
    return float4(layerRgb, layerAlpha);
}

[numthreads(8, 8, 1)]
void computeMain(uint3 globalId : SV_DispatchThreadID) {
    uint width, height;
    outputTexture.GetDimensions(width, height);
    float2 res = float2(width, height);
    if (globalId.x >= width || globalId.y >= height) return;
    
    float2 fragCoord = float2(globalId.xy);
    float2 uv = (fragCoord - 0.5 * res) / (res.x * 0.5);
    
    // Dynamic breathing and flowing
    float anim = _time * _animSpeed;
    float breath = 1.0 + sin(anim * PI / 15.0) * 0.015; // 30s breath cycle
    float2 p = uv / breath;
    
    // Exactly 30 seconds for a full cycle (TAU/2 = PI distance mapped into getColor)
    float color_flow = anim * (PI / 30.0);
    // Subtle crawl for the physical pencil strokes
    float stroke_flow = anim * 0.15;
    
    // ---- 3D Ribbon Branch Separation ----
    // Branch 1: Right half of the figure 8, overlaps FRONT at center
    float d1 = 1e10; float t1 = 0;
    for(int i = 0; i < CURVE_SAMPLES; i++) {
        float t = -PI/2.0 + PI * (float(i)/float(CURVE_SAMPLES - 1));
        float2 cp = curvePt(t);
        float d = dot(p - cp, p - cp);
        if(d < d1) { d1 = d; t1 = t; }
    }
    
    // Branch 2: Left half of the figure 8, overlaps BACK at center
    float d2 = 1e10; float t2 = 0;
    for(int i = 0; i < CURVE_SAMPLES; i++) {
        float t = PI/2.0 + PI * (float(i)/float(CURVE_SAMPLES - 1));
        float2 cp = curvePt(t);
        float d = dot(p - cp, p - cp);
        if(d < d2) { d2 = d; t2 = t; }
    }
    
    float z1 = curve3D(t1).z;
    float z2 = curve3D(t2).z;
    
    float t_front, t_back;
    if(z1 > z2) {
        t_front = t1; t_back = t2;
    } else {
        t_front = t2; t_back = t1;
    }
    
    // Render Back layer
    float4 backLayer = renderLayer(p, t_back, color_flow, stroke_flow, min(z1, z2));
    // Render Front layer
    float4 frontLayer = renderLayer(p, t_front, color_flow, stroke_flow, max(z1, z2));
    
    // Alpha compositing (Front over Back)
    // To get the pencil shadow effect, the front layer should slightly darken the back layer
    // where they overlap. Standard over operator handles this mostly, but we can add a slight drop shadow.
    float shadow = smoothstep(0.0, 0.4, frontLayer.a);
    backLayer.rgb *= (1.0 - shadow * 0.4); // Darken back layer by up to 40% under front layer
    
    float outAlpha = frontLayer.a + backLayer.a * (1.0 - frontLayer.a);
    float3 outRgb = float3(0);
    
    if (outAlpha > 0.0001) {
        // Standard alpha blend
        outRgb = (frontLayer.rgb * frontLayer.a + backLayer.rgb * backLayer.a * (1.0 - frontLayer.a)) / outAlpha;
    }
    
    // Central Starburst Core
    float dc = length(p);
    
    // Sharp, crisp 4-pointed star mimicking the reference
    // Reference star has very sharp main rays
    float cross_ray = exp(-abs(p.x) * 30.0 - abs(p.y) * 600.0) + exp(-abs(p.y) * 30.0 - abs(p.x) * 600.0);
    float diag_ray = exp(-abs(p.x + p.y) * 50.0 - abs(p.x - p.y) * 400.0) + exp(-abs(p.x - p.y) * 50.0 - abs(p.x + p.y) * 400.0);
    
    // Hard core
    float core = exp(-dc * 60.0);
    
    float star = (cross_ray * 2.0 + diag_ray * 0.5 + core * 2.0);
    
    // Add black hole at the very center (the sharp diamond cutout)
    float blackHoleDist = abs(p.x) + abs(p.y);
    float blackHole = smoothstep(0.010, 0.025, blackHoleDist); 
    star *= blackHole;
    
    // Subtle wide glow
    float glow = exp(-dc * 8.0) * 0.2;
    star += glow * smoothstep(0.3, 0.0, dc) * blackHole;
    
    // Bright white/yellow center star color with 30-second cyclic twinkling
    float twinkle = 1.0 + 0.15 * sin(anim * PI / 15.0 * 5.0); // Flashes 5 times per 30s cycle
    float3 starColor = float3(1.0, 0.98, 0.9) * star * twinkle;
    
    // Blend star (additive, but respecting the alpha to keep it visible on transparent)
    outRgb += starColor;
    outAlpha = saturate(outAlpha + star);
    
    // Post-processing
    if (outAlpha > 0.001) {
        // Boost contrast heavily to make pencil strokes pop like reference
        outRgb = smoothstep(0.05, 0.95, outRgb);
        
        // sRGB Gamma
        outRgb = pow(max(outRgb, 0.0), 1.0 / 2.2);
        // Pre-multiply for straight-alpha PNG export pipeline expectation
        outRgb *= outAlpha;
    }
    
    // Ensure we un-premultiply at the very end because view_logo.py expects straight alpha
    if (outAlpha > 0.001) {
        outRgb /= outAlpha;
    }
    
    outputTexture[globalId.xy] = float4(saturate(outRgb), outAlpha);
}
