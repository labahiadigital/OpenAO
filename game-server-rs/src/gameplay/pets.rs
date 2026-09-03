/// Pet type IDs correspond to NPC body IDs for visual rendering.
pub type PetTypeId = i32;

#[derive(Debug, Clone)]
pub struct Pet {
    pub pet_type: PetTypeId,
    pub name: String,
    pub level: i32,
    pub exp: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub active: bool,
}

impl Pet {
    pub fn new(pet_type: PetTypeId, name: String) -> Self {
        Self {
            pet_type,
            name,
            level: 1,
            exp: 0,
            hp: 50,
            max_hp: 50,
            active: false,
        }
    }

    pub fn gain_exp(&mut self, amount: i32) -> bool {
        self.exp += amount;
        let threshold = self.level * 100;
        if self.exp >= threshold {
            self.exp -= threshold;
            self.level += 1;
            self.max_hp += 10;
            self.hp = self.max_hp;
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}

/// Per-player pet manager. A player can own multiple pets but only one active.
#[derive(Debug, Clone, Default)]
pub struct PetManager {
    pub pets: Vec<Pet>,
}

impl PetManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pet(&mut self, pet: Pet) -> Result<(), &'static str> {
        if self.pets.len() >= 5 {
            return Err("No puedes tener mas de 5 mascotas.");
        }
        self.pets.push(pet);
        Ok(())
    }

    pub fn active_pet(&self) -> Option<&Pet> {
        self.pets.iter().find(|p| p.active)
    }

    pub fn active_pet_mut(&mut self) -> Option<&mut Pet> {
        self.pets.iter_mut().find(|p| p.active)
    }

    pub fn summon(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.pets.len() {
            return Err("Mascota no encontrada.");
        }
        for p in self.pets.iter_mut() {
            p.active = false;
        }
        if !self.pets[index].is_alive() {
            return Err("Tu mascota esta muerta. Curala primero.");
        }
        self.pets[index].active = true;
        Ok(())
    }

    pub fn dismiss(&mut self) {
        for p in self.pets.iter_mut() {
            p.active = false;
        }
    }

    pub fn release(&mut self, index: usize) -> Result<String, &'static str> {
        if index >= self.pets.len() {
            return Err("Mascota no encontrada.");
        }
        if self.pets[index].active {
            return Err("No puedes liberar una mascota activa. Despachala primero.");
        }
        let removed = self.pets.remove(index);
        Ok(removed.name)
    }

    pub fn heal_pet(&mut self, index: usize, amount: i32) -> Result<(), &'static str> {
        if index >= self.pets.len() {
            return Err("Mascota no encontrada.");
        }
        self.pets[index].heal(amount);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_summon_pet() {
        let mut mgr = PetManager::new();
        mgr.add_pet(Pet::new(10, "Lobo".into())).unwrap();
        assert!(mgr.active_pet().is_none());
        mgr.summon(0).unwrap();
        assert!(mgr.active_pet().is_some());
        assert_eq!(mgr.active_pet().unwrap().name, "Lobo");
    }

    #[test]
    fn dismiss_pet() {
        let mut mgr = PetManager::new();
        mgr.add_pet(Pet::new(10, "Lobo".into())).unwrap();
        mgr.summon(0).unwrap();
        mgr.dismiss();
        assert!(mgr.active_pet().is_none());
    }

    #[test]
    fn pet_level_up() {
        let mut pet = Pet::new(10, "Lobo".into());
        assert_eq!(pet.level, 1);
        let leveled = pet.gain_exp(100);
        assert!(leveled);
        assert_eq!(pet.level, 2);
        assert_eq!(pet.max_hp, 60);
    }

    #[test]
    fn pet_damage_and_heal() {
        let mut pet = Pet::new(10, "Lobo".into());
        pet.take_damage(30);
        assert_eq!(pet.hp, 20);
        pet.heal(10);
        assert_eq!(pet.hp, 30);
        pet.heal(100);
        assert_eq!(pet.hp, 50);
    }

    #[test]
    fn cannot_summon_dead_pet() {
        let mut mgr = PetManager::new();
        let mut pet = Pet::new(10, "Lobo".into());
        pet.hp = 0;
        mgr.add_pet(pet).unwrap();
        assert!(mgr.summon(0).is_err());
    }

    #[test]
    fn max_pets_limit() {
        let mut mgr = PetManager::new();
        for i in 0..5 {
            mgr.add_pet(Pet::new(i, format!("Pet{}", i))).unwrap();
        }
        assert!(mgr.add_pet(Pet::new(5, "Extra".into())).is_err());
    }

    #[test]
    fn release_pet() {
        let mut mgr = PetManager::new();
        mgr.add_pet(Pet::new(10, "Lobo".into())).unwrap();
        mgr.add_pet(Pet::new(11, "Oso".into())).unwrap();
        let name = mgr.release(0).unwrap();
        assert_eq!(name, "Lobo");
        assert_eq!(mgr.pets.len(), 1);
    }

    #[test]
    fn cannot_release_active_pet() {
        let mut mgr = PetManager::new();
        mgr.add_pet(Pet::new(10, "Lobo".into())).unwrap();
        mgr.summon(0).unwrap();
        assert!(mgr.release(0).is_err());
    }
}
