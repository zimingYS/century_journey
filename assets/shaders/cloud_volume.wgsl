// ============================================================================
// CenturyJourney - ARTShade Style Voxel Clouds
// Reference-based reconstruction
// ============================================================================

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
}

// ============================================================================
// Cloud Uniform
// ============================================================================

struct CloudVolumeExtension {
    time_seconds: f32,
    coverage: f32,
    night_factor: f32,
    twilight_glow: f32,

    cloud_min_y: f32,
    cloud_max_y: f32,

    wind_speed: f32,
    noise_scale: f32,

    cell_size: f32,
    density_threshold: f32,
    detail_strength: f32,
    visibility: f32,

    camera_position: vec4<f32>,
    sun_direction: vec4<f32>,
    wind_direction: vec4<f32>,

    tint_day: vec4<f32>,
    tint_night: vec4<f32>,
    tint_sunset: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> cloud: CloudVolumeExtension;

// ============================================================================
// Constants
// ============================================================================

const MAX_ABSOLUTE_STEPS: i32 = 1536;

const CLOUD_RENDER_DISTANCE: f32 = 320.0;

const CLOUD_SMOOTH_DOWN: f32 = 80.0;
const CLOUD_SMOOTH_UP: f32 = 75.0;

const SUN_SCATTER_POWER: f32 = 8.0;
const SUN_SCATTER_STRENGTH: f32 = 0.55;

// ============================================================================
// 2D Hash
// ============================================================================

fn hash21(p: vec2<f32>) -> f32 {
    let n =
        sin(
            dot(
                p,
                vec2<f32>(
                    127.1,
                    311.7,
                ),
            ),
        ) *
        43758.5453;

    return fract(n);
}

// ============================================================================
// 2D Value Noise
// ============================================================================

fn value_noise_2d(
    p: vec2<f32>,
) -> f32 {
    let i =
        floor(p);

    let f =
        fract(p);

    let u =
        f *
        f *
        (
            3.0 -
            2.0 *
            f
        );

    let n00 =
        hash21(
            i +
            vec2<f32>(
                0.0,
                0.0,
            ),
        );

    let n10 =
        hash21(
            i +
            vec2<f32>(
                1.0,
                0.0,
            ),
        );

    let n01 =
        hash21(
            i +
            vec2<f32>(
                0.0,
                1.0,
            ),
        );

    let n11 =
        hash21(
            i +
            vec2<f32>(
                1.0,
                1.0,
            ),
        );

    let x0 =
        mix(
            n00,
            n10,
            u.x,
        );

    let x1 =
        mix(
            n01,
            n11,
            u.x,
        );

    return mix(
        x0,
        x1,
        u.y,
    );
}

// ============================================================================
// Cloud Noise
//
// Reference structure:
//
// octave 1 : 0.5000
// octave 2 : 0.2500
// octave 3 : 0.1250
// octave 4 : 0.0625
//
// This is intentionally kept close to the uploaded reference.
// ============================================================================

fn cloud_noise(
    p: vec2<f32>,
    scale: f32,
    layer_idx: i32,
) -> f32 {

    var moved =
        p *
        scale;

    if (layer_idx == 2) {
        moved += vec2<f32>(
            5333.0,
            2187.0,
        );
    } else if (layer_idx == 3) {
        moved += vec2<f32>(
            -2814.0,
            6942.0,
        );
    }

    var value =
        0.0;

    // ------------------------------------------------------------------------
    // Octave 1
    // ------------------------------------------------------------------------

    value +=
        value_noise_2d(
            moved,
        ) *
        0.5000;

    // ------------------------------------------------------------------------
    // Octave 2
    // ------------------------------------------------------------------------

    moved =
        moved *
        2.02 +
        vec2<f32>(
            12.3,
            12.3,
        );

    value +=
        value_noise_2d(
            moved,
        ) *
        0.2500;

    // ------------------------------------------------------------------------
    // Octave 3
    // ------------------------------------------------------------------------

    moved =
        moved *
        2.03 +
        vec2<f32>(
            45.6,
            45.6,
        );

    value +=
        value_noise_2d(
            moved,
        ) *
        0.1250;

    // ------------------------------------------------------------------------
    // Octave 4
    // ------------------------------------------------------------------------

    moved =
        moved *
        2.01 +
        vec2<f32>(
            78.9,
            78.9,
        );

    value +=
        value_noise_2d(
            moved,
        ) *
        0.0625;

    return value;
}

// ============================================================================
// Cloud Map
//
// Closely follows the uploaded reference's:
//     f += 0.5 / 0.25 / 0.125 / 0.0625
//     f = smoothstep(0.1, 0.9, f)
//     smoothstep(cutoff - shore, cutoff + shore, f)
// ============================================================================

fn cloud_map(
    world_xz: vec2<f32>,
    time: f32,
    coverage: f32,
    narrowness: f32,
    layer_idx: i32,
    ray_distance: f32,
) -> f32 {

    // ------------------------------------------------------------------------
    // Wind
    // ------------------------------------------------------------------------

    let wind =
        cloud.wind_direction.xy *
        time *
        cloud.wind_speed;

    var moved =
        world_xz +
        wind;

    // ------------------------------------------------------------------------
    // Layer-specific transform
    // ------------------------------------------------------------------------

    var scale_1 =
        1.0;

    var scale_2 =
        1.0;

    let mix_factor =
        0.5;

    if (layer_idx == 2) {

        moved +=
            vec2<f32>(
                5333.0,
                2187.0,
            );

        scale_1 =
            1.31;

        scale_2 =
            1.55;

    } else if (layer_idx == 3) {

        moved +=
            vec2<f32>(
                -2814.0,
                6942.0,
            );

        scale_1 =
            1.55;

        scale_2 =
            1.85;
    }

    // ------------------------------------------------------------------------
    // Noise scale
    //
    // Keep the user-facing noise_scale parameter.
    // narrowness remains the fallback.
    // ------------------------------------------------------------------------

    let fallback_scale =
        max(
            narrowness *
            0.15,
            0.0001,
        );

    let base_scale =
        max(
            select(
                fallback_scale,
                cloud.noise_scale,
                cloud.noise_scale >
                0.0001,
            ),
            0.0001,
        );

    // ------------------------------------------------------------------------
    // Morph
    //
    // Keep spatial motion stable and slow.
    // ------------------------------------------------------------------------

    let wind_len =
        max(
            length(
                cloud.wind_direction.xy,
            ),
            0.001,
        );

    let wind_dir =
        cloud.wind_direction.xy /
        wind_len;

    let morph_time =
        time *
        0.035;

    let morph_offset =
        wind_dir *
        morph_time *
        15.0;

    let p1 =
        (
            moved +
            morph_offset
        ) *
        base_scale *
        scale_1;

    let p2 =
        (
            moved -
            morph_offset
        ) *
        base_scale *
        scale_2;

    // ------------------------------------------------------------------------
    // Four-octave FBM
    // ------------------------------------------------------------------------

    var q1 =
        p1;

    var q2 =
        p2;

    var sample1 =
        value_noise_2d(
            q1,
        );

    var sample2 =
        value_noise_2d(
            q2,
        );

    var density =
        mix(
            sample1,
            sample2,
            mix_factor,
        ) *
        0.5000;

    // ------------------------------------------------------------------------
    // Octave 2
    // ------------------------------------------------------------------------

    q1 =
        q1 *
        2.02 +
        vec2<f32>(
            12.3,
            12.3,
        );

    q2 =
        q2 *
        2.02 +
        vec2<f32>(
            12.3,
            12.3,
        );

    sample1 =
        value_noise_2d(
            q1,
        );

    sample2 =
        value_noise_2d(
            q2,
        );

    density +=
        mix(
            sample1,
            sample2,
            mix_factor,
        ) *
        0.2500;

    // ------------------------------------------------------------------------
    // Coverage
    //
    // Same overall idea as the reference:
    //
    // currentCoverage
    //     ↓
    // cutoff
    //     ↓
    // early empty-space rejection
    // ------------------------------------------------------------------------

    let coverage_value =
        clamp(
            coverage,
            0.0,
            1.0,
        );

    let threshold_offset =
        clamp(
            cloud.density_threshold,
            -0.15,
            0.15,
        );

    let shore_width =
        mix(
            0.001,
            0.04,
            clamp(
                cloud.detail_strength,
                0.0,
                1.0,
            ),
        );

    let cutoff =
        1.0 -
        (
            coverage_value *
            1.2
        ) +
        threshold_offset;

    let early_cutoff =
        clamp(
            cutoff -
            shore_width,
            0.0,
            1.0,
        );

    if (
        density +
        0.1875 <
        (
            0.1 +
            0.8 *
            sqrt(
                max(
                    early_cutoff,
                    0.0,
                ) /
                3.0,
            )
        )
    ) {
        return 0.0;
    }

    // ------------------------------------------------------------------------
    // Octave 3
    // ------------------------------------------------------------------------

    q1 =
        q1 *
        2.03 +
        vec2<f32>(
            45.6,
            45.6,
        );

    q2 =
        q2 *
        2.03 +
        vec2<f32>(
            45.6,
            45.6,
        );

    sample1 =
        value_noise_2d(
            q1,
        );

    sample2 =
        value_noise_2d(
            q2,
        );

    density +=
        mix(
            sample1,
            sample2,
            mix_factor,
        ) *
        0.1250;

    // ------------------------------------------------------------------------
    // Octave 4
    // ------------------------------------------------------------------------

    q1 =
        q1 *
        2.01 +
        vec2<f32>(
            78.9,
            78.9,
        );

    q2 =
        q2 *
        2.01 +
        vec2<f32>(
            78.9,
            78.9,
        );

    sample1 =
        value_noise_2d(
            q1,
        );

    sample2 =
        value_noise_2d(
            q2,
        );

    density +=
        mix(
            sample1,
            sample2,
            mix_factor,
        ) *
        0.0625;

    // ------------------------------------------------------------------------
    // Reference density shaping
    // ------------------------------------------------------------------------

    density =
        smoothstep(
            0.1,
            0.9,
            density,
        );

    // ------------------------------------------------------------------------
    // Final cloud edge
    // ------------------------------------------------------------------------

    return smoothstep(
        cutoff -
        shore_width,
        cutoff +
        shore_width,
        density,
    );
}

// ============================================================================
// Ray / Cloud Slab Intersection
// ============================================================================

fn ray_slab_intersect(
    origin: vec3<f32>,
    dir: vec3<f32>,
    y_bottom: f32,
    y_top: f32,
) -> vec2<f32> {

    if (
        abs(dir.y) <
        0.0005
    ) {

        if (
            origin.y >= y_bottom &&
            origin.y <= y_top
        ) {
            return vec2<f32>(
                0.0,
                1e6,
            );
        }

        return vec2<f32>(
            -1.0,
            -1.0,
        );
    }

    let t0 =
        (
            y_bottom -
            origin.y
        ) /
        dir.y;

    let t1 =
        (
            y_top -
            origin.y
        ) /
        dir.y;

    var t_near =
        min(
            t0,
            t1,
        );

    var t_far =
        max(
            t0,
            t1,
        );

    if (
        t_far <
        0.0
    ) {
        return vec2<f32>(
            -1.0,
            -1.0,
        );
    }

    t_near =
        max(
            t_near,
            0.0,
        );

    if (
        t_near >=
        t_far
    ) {
        return vec2<f32>(
            -1.0,
            -1.0,
        );
    }

    return vec2<f32>(
        t_near,
        t_far,
    );
}

// ============================================================================
// Trace Layer
//
// Main structure copied from the reference design:
//
// 1. cameraInside
// 2. initial cell density
// 3. positive density delta
// 4. actual cell boundary hit = curHitT
// 5. hitPos-based vertical shaping
// 6. alpha accumulation
//
// The important difference from your old version is that density is no longer
// sampled at arbitrary segment centers for the actual cloud contribution.
// ============================================================================

fn trace_layer(
    cam_pos: vec3<f32>,
    dir: vec3<f32>,
    t_near: f32,
    t_far: f32,
    layer_bottom: f32,
    layer_top: f32,
    coverage: f32,
    cell_size: f32,
    narrowness: f32,
    time: f32,
    cloud_render_dist: f32,
    smooth_down: f32,
    smooth_up: f32,
    layer_idx: i32,
    top_color: vec3<f32>,
    bottom_color: vec3<f32>,
    max_dist: f32,
) -> vec4<f32> {

    let safe_cell =
        max(
            cell_size,
            0.001,
        );

    let max_steps =
        min(
            i32(
                cloud_render_dist /
                (
                    safe_cell *
                    0.7071
                )
            ) +
            2,
            MAX_ABSOLUTE_STEPS,
        );

    let layer_render_dist =
        min(
            cloud_render_dist,
            f32(max_steps) *
            safe_cell *
            0.7071,
        );

    if (
        t_near >= t_far ||
        t_near >= layer_render_dist ||
        t_near >= max_dist
    ) {
        return vec4<f32>(0.0);
    }

    // ------------------------------------------------------------------------
    // Wind
    // ------------------------------------------------------------------------

    let wind =
        cloud.wind_direction.xy *
        time *
        cloud.wind_speed;

    // ------------------------------------------------------------------------
    // Entry
    // ------------------------------------------------------------------------

    let entry_pos =
        cam_pos +
        dir *
        t_near;

    let start_xz =
        entry_pos.xz +
        wind;

    let dir_xz =
        dir.xz;

    // ------------------------------------------------------------------------
    // Camera inside cloud
    // ------------------------------------------------------------------------

    let camera_inside =
        !(
            cam_pos.y <
            layer_bottom ||
            cam_pos.y >
            layer_top
        );

    // ------------------------------------------------------------------------
    // Colors
    // ------------------------------------------------------------------------

    let sky_brightness =
        clamp(
            length(
                top_color,
            ) *
            1.5,
            0.05,
            1.0,
        );

    let base_cloud_color =
        vec3<f32>(
            sky_brightness,
            sky_brightness,
            sky_brightness,
        );

    let pre_top_color =
        mix(
            base_cloud_color,
            top_color,
            0.30,
        );

    let pre_bottom_color =
        mix(
            base_cloud_color *
            0.55,
            bottom_color,
            0.85,
        );

    var acc_color =
        vec3<f32>(0.0);

    var acc_alpha =
        0.0;

    // ------------------------------------------------------------------------
    // Initial cell
    // ------------------------------------------------------------------------

    let initial_cell =
        floor(
            start_xz /
            safe_cell,
        );

    let initial_center =
        (
            initial_cell +
            vec2<f32>(
                0.5,
                0.5,
            )
        ) *
        safe_cell;

    let initial_world_xz =
        initial_center -
        wind;

    var prev_density =
        cloud_map(
            initial_world_xz,
            time,
            coverage,
            narrowness,
            layer_idx,
            t_near,
        );

    // ------------------------------------------------------------------------
    // Entering cloud from outside
    //
    // This is one of the reference implementation's most important details.
    // ------------------------------------------------------------------------

    if (
        !camera_inside &&
        prev_density > 0.001
    ) {

        let hit_dist =
            length(
                entry_pos.xz -
                cam_pos.xz,
            );

        let distance_fade =
            1.0 -
            smoothstep(
                layer_render_dist *
                0.6,
                layer_render_dist,
                hit_dist,
            );

        if (
            distance_fade >
            0.001
        ) {

            let local_y =
                clamp(
                    (
                        entry_pos.y -
                        layer_bottom
                    ) /
                    max(
                        layer_top -
                        layer_bottom,
                        0.001,
                    ),
                    0.0,
                    1.0,
                );

            let color_t =
                smoothstep(
                    0.0,
                    1.0,
                    local_y,
                );

            let col =
                mix(
                    pre_bottom_color,
                    pre_top_color,
                    color_t,
                );

            let top_edge =
                smooth_up *
                2.0;

            let top_alpha =
                1.0 -
                smoothstep(
                    top_edge -
                    1.0,
                    top_edge,
                    local_y,
                );

            let bottom_edge =
                (
                    1.0 -
                    smooth_down
                ) *
                2.0 -
                1.0;

            let bottom_alpha =
                smoothstep(
                    bottom_edge,
                    bottom_edge +
                    1.0,
                    local_y,
                );

            let base_alpha =
                top_alpha *
                bottom_alpha;

            let step_alpha =
                clamp(
                    base_alpha *
                    distance_fade *
                    prev_density,
                    0.0,
                    1.0,
                );

            let remaining =
                1.0 -
                acc_alpha;

            acc_color +=
                col *
                step_alpha *
                remaining;

            acc_alpha +=
                step_alpha *
                remaining;
        }

    } else if (camera_inside) {

        prev_density =
            cloud_map(
                initial_world_xz,
                time,
                coverage,
                narrowness,
                layer_idx,
                0.0,
            );
    }

    // ------------------------------------------------------------------------
    // Almost vertical ray
    // ------------------------------------------------------------------------

    if (
        length(dir_xz) <
        0.0005
    ) {
        return vec4<f32>(
            acc_color,
            acc_alpha,
        );
    }

    // ------------------------------------------------------------------------
    // DDA initialization
    // ------------------------------------------------------------------------

    var cell_idx =
        floor(
            start_xz /
            safe_cell,
        );

    var step_dir =
        vec2<f32>(
            0.0,
            0.0,
        );

    if (
        dir_xz.x >
        0.0
    ) {
        step_dir.x =
            1.0;
    } else if (
        dir_xz.x <
        0.0
    ) {
        step_dir.x =
            -1.0;
    }

    if (
        dir_xz.y >
        0.0
    ) {
        step_dir.y =
            1.0;
    } else if (
        dir_xz.y <
        0.0
    ) {
        step_dir.y =
            -1.0;
    }

    let inv_dir_x =
        select(
            1e20,
            1.0 /
            dir_xz.x,
            abs(
                dir_xz.x
            ) >
            0.0001,
        );

    let inv_dir_y =
        select(
            1e20,
            1.0 /
            dir_xz.y,
            abs(
                dir_xz.y
            ) >
            0.0001,
        );

    let t_delta_x =
        abs(
            safe_cell *
            inv_dir_x,
        );

    let t_delta_y =
        abs(
            safe_cell *
            inv_dir_y,
        );

    var t_max_x =
        1e20;

    var t_max_y =
        1e20;

    if (
        abs(
            dir_xz.x
        ) >
        0.0001
    ) {

        let next_x =
            (
                cell_idx.x +
                select(
                    0.0,
                    1.0,
                    step_dir.x >
                    0.0,
                )
            ) *
            safe_cell;

        t_max_x =
            (
                next_x -
                start_xz.x
            ) *
            inv_dir_x;
    }

    if (
        abs(
            dir_xz.y
        ) >
        0.0001
    ) {

        let next_y =
            (
                cell_idx.y +
                select(
                    0.0,
                    1.0,
                    step_dir.y >
                    0.0,
                )
            ) *
            safe_cell;

        t_max_y =
            (
                next_y -
                start_xz.y
            ) *
            inv_dir_y;
    }

    let t_limit =
        min(
            t_far,
            min(
                cloud_render_dist,
                max_dist,
            ),
        );

    // ------------------------------------------------------------------------
    // Main DDA
    // ------------------------------------------------------------------------

    for (
        var i: i32 = 0;
        i < MAX_ABSOLUTE_STEPS;
        i = i + 1
    ) {

        if (
            i >=
            max_steps ||
            acc_alpha >=
            0.99
        ) {
            break;
        }

        // --------------------------------------------------------------------
        // Current voxel/cell density
        // --------------------------------------------------------------------

        let world_cell =
            (
                cell_idx +
                vec2<f32>(
                    0.5,
                    0.5,
                )
            ) *
            safe_cell -
            wind;

        let block_density =
            cloud_map(
                world_cell,
                time,
                coverage,
                narrowness,
                layer_idx,
                t_near,
            );

        // --------------------------------------------------------------------
        // Positive density change
        //
        // Reference behavior.
        // --------------------------------------------------------------------

        let delta_density =
            max(
                0.0,
                block_density -
                prev_density,
            );

        if (
            delta_density >
            0.001
        ) {

            // ---------------------------------------------------------------
            // Actual entry point into this cell
            // ---------------------------------------------------------------

            let cell_min =
                cell_idx *
                safe_cell;

            let cell_max =
                cell_min +
                safe_cell;

            var t_entry_x =
                0.0;

            var t_entry_y =
                0.0;

            if (
                abs(
                    dir_xz.x
                ) >
                0.0001
            ) {

                let boundary_x =
                    select(
                        cell_max.x,
                        cell_min.x,
                        dir_xz.x >
                        0.0,
                    );

                t_entry_x =
                    (
                        boundary_x -
                        start_xz.x
                    ) *
                    inv_dir_x;
            }

            if (
                abs(
                    dir_xz.y
                ) >
                0.0001
            ) {

                let boundary_y =
                    select(
                        cell_max.y,
                        cell_min.y,
                        dir_xz.y >
                        0.0,
                    );

                t_entry_y =
                    (
                        boundary_y -
                        start_xz.y
                    ) *
                    inv_dir_y;
            }

            let cur_hit_t =
                t_near +
                max(
                    max(
                        t_entry_x,
                        t_entry_y,
                    ),
                    0.0,
                );

            if (
                cur_hit_t >
                t_limit
            ) {
                break;
            }

            // ---------------------------------------------------------------
            // Actual world position
            // ---------------------------------------------------------------

            let hit_pos =
                cam_pos +
                dir *
                cur_hit_t;

            let hit_dist =
                length(
                    hit_pos.xz -
                    cam_pos.xz,
                );

            // ---------------------------------------------------------------
            // Distance fade
            // ---------------------------------------------------------------

            let distance_fade =
                1.0 -
                smoothstep(
                    layer_render_dist *
                    0.6,
                    layer_render_dist,
                    hit_dist,
                );

            if (
                distance_fade >
                0.001
            ) {

                // -----------------------------------------------------------
                // Vertical coordinate
                // -----------------------------------------------------------

                let layer_height =
                    max(
                        layer_top -
                        layer_bottom,
                        0.001,
                    );

                let local_y =
                    clamp(
                        (
                            hit_pos.y -
                            layer_bottom
                        ) /
                        layer_height,
                        0.0,
                        1.0,
                    );

                // -----------------------------------------------------------
                // Vertical color
                // -----------------------------------------------------------

                let color_t =
                    smoothstep(
                        0.0,
                        1.0,
                        local_y,
                    );

                let col =
                    mix(
                        pre_bottom_color,
                        pre_top_color,
                        color_t,
                    );

                // -----------------------------------------------------------
                // Top / bottom edges
                // -----------------------------------------------------------

                let top_edge =
                    smooth_up *
                    2.0;

                let top_alpha =
                    1.0 -
                    smoothstep(
                        top_edge -
                        1.0,
                        top_edge,
                        local_y,
                    );

                let bottom_edge =
                    (
                        1.0 -
                        smooth_down
                    ) *
                    2.0 -
                    1.0;

                let bottom_alpha =
                    smoothstep(
                        bottom_edge,
                        bottom_edge +
                        1.0,
                        local_y,
                    );

                let base_alpha =
                    top_alpha *
                    bottom_alpha;

                // -----------------------------------------------------------
                // IMPORTANT:
                //
                // No old fixed CLOUD_EXTINCTION multiplication.
                // No segment-length multiplication.
                //
                // Reference uses:
                //
                // baseAlpha * distFade * deltaDensity
                // -----------------------------------------------------------

                let step_alpha =
                    clamp(
                        base_alpha *
                        distance_fade *
                        delta_density,
                        0.0,
                        1.0,
                    );

                let remaining =
                    1.0 -
                    acc_alpha;

                acc_color +=
                    col *
                    step_alpha *
                    remaining;

                acc_alpha +=
                    step_alpha *
                    remaining;
            }
        }

        // --------------------------------------------------------------------
        // Store previous density
        // --------------------------------------------------------------------

        prev_density =
            block_density;

        // --------------------------------------------------------------------
        // Advance DDA
        // --------------------------------------------------------------------

        if (
            t_max_x <
            t_max_y
        ) {

            cell_idx.x +=
                step_dir.x;

            if (
                t_near +
                t_max_x >
                t_limit
            ) {
                break;
            }

            t_max_x +=
                t_delta_x;

        } else {

            cell_idx.y +=
                step_dir.y;

            if (
                t_near +
                t_max_y >
                t_limit
            ) {
                break;
            }

            t_max_y +=
                t_delta_y;
        }
    }

    return vec4<f32>(
        acc_color,
        acc_alpha,
    );
}

// ============================================================================
// Fragment
// ============================================================================

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {

    var pbr_input =
        pbr_input_from_standard_material(
            in,
            is_front,
        );

    // ------------------------------------------------------------------------
    // Camera ray
    // ------------------------------------------------------------------------

    let ray_origin =
        cloud.camera_position.xyz;

    let ray_dir =
        normalize(
            in.world_position.xyz -
            ray_origin,
        );

    // ------------------------------------------------------------------------
    // Cloud layers
    // ------------------------------------------------------------------------

    let base_altitude =
        cloud.cloud_min_y;

    let layer_thickness =
        max(
            cloud.cloud_max_y -
            cloud.cloud_min_y,
            0.001,
        );

    let layer_gap =
        layer_thickness *
        0.45;

    let cell_size =
        max(
            cloud.cell_size,
            0.001,
        );

    let render_dist =
        CLOUD_RENDER_DISTANCE;

    // ------------------------------------------------------------------------
    // Sky tint
    // ------------------------------------------------------------------------

    var cloud_tint =
        mix(
            cloud.tint_day.rgb,
            cloud.tint_night.rgb,
            clamp(
                cloud.night_factor,
                0.0,
                1.0,
            ),
        );

    cloud_tint =
        mix(
            cloud_tint,
            cloud.tint_sunset.rgb,
            clamp(
                cloud.twilight_glow *
                0.5,
                0.0,
                1.0,
            ),
        );

    let top_color =
        cloud_tint;

    let bottom_color =
        cloud_tint *
        0.42;

    let coverage =
        clamp(
            cloud.coverage,
            0.0,
            1.0,
        );

    // ------------------------------------------------------------------------
    // Layer 1
    // ------------------------------------------------------------------------

    let l1_bottom =
        base_altitude;

    let l1_top =
        l1_bottom +
        layer_thickness;

    let d1 =
        ray_slab_intersect(
            ray_origin,
            ray_dir,
            l1_bottom,
            l1_top,
        );

    var cloud1 =
        vec4<f32>(0.0);

    var depth1 =
        1e30;

    if (
        d1.y >
        0.0 &&
        d1.x <
        d1.y
    ) {

        cloud1 =
            trace_layer(
                ray_origin,
                ray_dir,
                d1.x,
                d1.y,
                l1_bottom,
                l1_top,
                coverage,
                cell_size,
                0.07,
                cloud.time_seconds,
                render_dist,
                CLOUD_SMOOTH_DOWN /
                100.0,
                CLOUD_SMOOTH_UP /
                100.0,
                1,
                top_color,
                bottom_color,
                render_dist,
            );

        depth1 =
            d1.x;
    }

    // ------------------------------------------------------------------------
    // Layer 2
    // ------------------------------------------------------------------------

    let l2_bottom =
        l1_top +
        layer_gap;

    let l2_top =
        l2_bottom +
        layer_thickness;

    let d2 =
        ray_slab_intersect(
            ray_origin,
            ray_dir,
            l2_bottom,
            l2_top,
        );

    var cloud2 =
        vec4<f32>(0.0);

    var depth2 =
        1e30;

    if (
        d2.y >
        0.0 &&
        d2.x <
        d2.y
    ) {

        cloud2 =
            trace_layer(
                ray_origin,
                ray_dir,
                d2.x,
                d2.y,
                l2_bottom,
                l2_top,
                coverage *
                0.78,
                cell_size,
                0.07,
                cloud.time_seconds,
                render_dist,
                CLOUD_SMOOTH_DOWN /
                100.0,
                CLOUD_SMOOTH_UP /
                100.0,
                2,
                top_color,
                bottom_color,
                render_dist,
            );

        depth2 =
            d2.x;
    }

    // ------------------------------------------------------------------------
    // Layer 3
    // ------------------------------------------------------------------------

    let l3_bottom =
        l2_top +
        layer_gap;

    let l3_top =
        l3_bottom +
        layer_thickness;

    let d3 =
        ray_slab_intersect(
            ray_origin,
            ray_dir,
            l3_bottom,
            l3_top,
        );

    var cloud3 =
        vec4<f32>(0.0);

    var depth3 =
        1e30;

    if (
        d3.y >
        0.0 &&
        d3.x <
        d3.y
    ) {

        cloud3 =
            trace_layer(
                ray_origin,
                ray_dir,
                d3.x,
                d3.y,
                l3_bottom,
                l3_top,
                coverage *
                0.58,
                cell_size,
                0.07,
                cloud.time_seconds,
                render_dist,
                CLOUD_SMOOTH_DOWN /
                100.0,
                CLOUD_SMOOTH_UP /
                100.0,
                3,
                top_color,
                bottom_color,
                render_dist,
            );

        depth3 =
            d3.x;
    }

    // ------------------------------------------------------------------------
    // Sun Scatter
    // ------------------------------------------------------------------------

    let sun_dir =
        normalize(
            cloud.sun_direction.xyz,
        );

    let light_dot =
        max(
            dot(
                ray_dir,
                sun_dir,
            ),
            0.0,
        );

    let scatter =
        pow(
            light_dot,
            SUN_SCATTER_POWER,
        ) *
        SUN_SCATTER_STRENGTH;

    if (
        cloud1.a >
        0.0
    ) {
        cloud1 =
            vec4<f32>(
                cloud1.rgb +
                cloud_tint *
                scatter *
                cloud1.a,
                cloud1.a,
            );
    }

    if (
        cloud2.a >
        0.0
    ) {
        cloud2 =
            vec4<f32>(
                cloud2.rgb +
                cloud_tint *
                scatter *
                cloud2.a,
                cloud2.a,
            );
    }

    if (
        cloud3.a >
        0.0
    ) {
        cloud3 =
            vec4<f32>(
                cloud3.rgb +
                cloud_tint *
                scatter *
                cloud3.a,
                cloud3.a,
            );
    }

    // ------------------------------------------------------------------------
    // Sort layers: far -> near
    // ------------------------------------------------------------------------

    var colors =
        array<vec4<f32>, 3>(
            cloud1,
            cloud2,
            cloud3,
        );

    var depths =
        array<f32, 3>(
            depth1,
            depth2,
            depth3,
        );

    for (
        var i: i32 = 0;
        i < 2;
        i = i + 1
    ) {

        for (
            var j: i32 = 0;
            j < 2 - i;
            j = j + 1
        ) {

            if (
                depths[j] <
                depths[j + 1]
            ) {

                let temp_d =
                    depths[j];

                depths[j] =
                    depths[j + 1];

                depths[j + 1] =
                    temp_d;

                let temp_c =
                    colors[j];

                colors[j] =
                    colors[j + 1];

                colors[j + 1] =
                    temp_c;
            }
        }
    }

    // ------------------------------------------------------------------------
    // Composite
    // ------------------------------------------------------------------------

    var final_rgb =
        vec3<f32>(0.0);

    var final_alpha =
        0.0;

    for (
        var i: i32 = 0;
        i < 3;
        i = i + 1
    ) {

        let layer =
            colors[i];

        if (
            layer.a >
            0.0
        ) {

            final_rgb =
                final_rgb *
                (
                    1.0 -
                    layer.a
                ) +
                layer.rgb;

            final_alpha =
                final_alpha *
                (
                    1.0 -
                    layer.a
                ) +
                layer.a;
        }
    }

    // ------------------------------------------------------------------------
    // Empty
    // ------------------------------------------------------------------------

    if (
        final_alpha <=
        0.001
    ) {
        discard;
    }

    // ------------------------------------------------------------------------
    // Visibility
    // ------------------------------------------------------------------------

    let visibility =
        clamp(
            cloud.visibility,
            0.0,
            1.0,
        );

    let coverage_alpha =
        smoothstep(
            0.0,
            0.05,
            coverage,
        );

    let final_alpha_output =
        final_alpha *
        max(
            cloud.tint_day.a,
            0.0,
        ) *
        visibility *
        coverage_alpha;

    // ------------------------------------------------------------------------
    // PBR
    // ------------------------------------------------------------------------

    pbr_input.material.base_color =
        vec4<f32>(
            final_rgb,
            final_alpha_output,
        );

    pbr_input.material.emissive =
        vec4<f32>(
            vec3<f32>(0.0),
            1.0,
        );

    var out: FragmentOutput;

    out.color =
        apply_pbr_lighting(
            pbr_input,
        );

    out.color =
        main_pass_post_lighting_processing(
            pbr_input,
            out.color,
        );

    return out;
}