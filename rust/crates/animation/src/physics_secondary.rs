//! Modul Dynamic Secondary Motion & Principles of Animation (Squash & Stretch, Mass-Spring Dynamics)

use crate::pose::StickmanPose;

#[derive(Debug, Clone, Copy)]
pub struct SpringState {
    pub position: f64,
    pub velocity: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl SpringState {
    pub fn new(stiffness: f64, damping: f64) -> Self {
        Self {
            position: 0.0,
            velocity: 0.0,
            stiffness,
            damping,
        }
    }

    /// Step simulasi spring menggunakan Euler / Verlet integration
    pub fn update(&mut self, target: f64, dt: f64) -> f64 {
        let force = (target - self.position) * self.stiffness;
        let damping_force = -self.velocity * self.damping;
        let accel = force + damping_force;

        self.velocity += accel * dt;
        self.position += self.velocity * dt;
        self.position
    }
}

pub struct SecondaryPhysicsEngine {
    pub hair_spring: SpringState,
    pub cloth_spring: SpringState,
    pub last_pos_y: f64,
    pub last_vel_y: f64,
}

impl SecondaryPhysicsEngine {
    pub fn new() -> Self {
        Self {
            hair_spring: SpringState::new(120.0, 12.0),
            cloth_spring: SpringState::new(90.0, 10.0),
            last_pos_y: 0.0,
            last_vel_y: 0.0,
        }
    }

    /// Menghitung Squash & Stretch dinamis serta inersia pergerakan rambut & baju
    pub fn apply_physics(&mut self, pose: &mut StickmanPose, dt: f64) {
        if dt <= 0.0 {
            return;
        }

        // Hitung Kecepatan dan Akselerasi Vertikal
        let current_y = pose.pos_y + pose.body_y;
        let vel_y = (current_y - self.last_pos_y) / dt;
        let accel_y = (vel_y - self.last_vel_y) / dt;

        self.last_pos_y = current_y;
        self.last_vel_y = vel_y;

        // 1. Squash & Stretch (Prinsip Animasi Perfilman)
        // Saat akselerasi vertikal positif (meloncat): Stretch tinggi (squash_y > 1.0, stretch_x < 1.0)
        // Saat akselerasi vertikal negatif tajam (mendarat/benturan): Squash gepeng (squash_y < 1.0, stretch_x > 1.0)
        let stretch_factor = (1.0 + vel_y * 0.15 + accel_y * 0.02).clamp(0.65, 1.45);
        let squash_factor = 1.0 / stretch_factor.sqrt(); // Mempertahankan volume visual (Volume Conservation)

        pose.squash_y = stretch_factor;
        pose.stretch_x = squash_factor;

        // 2. Mass-Spring Physics untuk Rambut & Baju (Inersia Pergerakan Utama)
        let target_hair_inertia = -vel_y * 0.2 - pose.body_tilt * 0.3;
        let target_cloth_inertia = -vel_y * 0.3 - pose.body_tilt * 0.4;

        pose.secondary_hair_sway = self.hair_spring.update(target_hair_inertia, dt);
        pose.secondary_cloth_sway = self.cloth_spring.update(target_cloth_inertia, dt);
    }
}
