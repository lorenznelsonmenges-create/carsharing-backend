// Consts for cars (kilometer)
const MAINTENANCE_KM: u32 = 5000;
const TUV_KM: u32 = 15000;
// Consts for cars (days)
const MAINTENANCE_DAYS: u32 = 2;
const TUV_DAYS: u32 = 3;
// Consts for retirement
const MAX_AGE_DAYS: u32 = 3650;
const MAX_KM: u32 = 200000;
const MAX_RENTALS: u32 = 500;

use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PersonStatus {
    Active,
    Blocked,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CarStatus {
    Available,
    Rented,
    Maintenance(u32),
    Tuv(u32),
    Retired,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub identifier: String,          // CHANGED: &'a str -> String + pub (Frontend braucht Zugriff)
    pub license_valid_days: u32,     // CHANGED: pub (optional, aber praktisch fürs Frontend)
    pub status: PersonStatus,        // CHANGED: pub
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Car {
    pub identifier: String,          // CHANGED: &'a str -> String + pub
    pub mileage: u32,                // CHANGED: pub
    pub status: CarStatus,           // CHANGED: pub
    pub age_days: u32,               // CHANGED: pub
    pub rental_count: u32,           // CHANGED: pub
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reservation {
    pub person_id: String,           // CHANGED: &'a str -> String + pub
    pub car_id: String,              // CHANGED: &'a str -> String + pub
    pub priority: u32,               // CHANGED: pub
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarSharing {
    pub persons: Vec<Person>,                 // CHANGED: Vec<Person<'a>> -> Vec<Person>
    pub cars: Vec<Car>,                       // CHANGED: Vec<Car<'a>> -> Vec<Car>
    pub rentals: Vec<(String, String)>,       // CHANGED: Vec<(&'a str,&'a str)> -> Vec<(String,String)>
    pub reservations: Vec<Reservation>,       // CHANGED: Vec<Reservation<'a>> -> Vec<Reservation>
    pub current_day: u32,                     // CHANGED: pub (optional)
}

pub trait CarSharingService {
    // Personen
    fn register_person(&mut self, p: Person) -> bool;                 // CHANGED: Person<'a> -> Person
    fn unregister_person(&mut self, identifier: &str) -> bool;
    fn renew_license(&mut self, identifier: &str, new_valid_days: u32) -> bool;
    fn get_person_status(&self, identifier: &str) -> Option<PersonStatus>;

    // Autos
    fn register_car(&mut self, c: Car) -> bool;                       // CHANGED: Car<'a> -> Car
    fn unregister_car(&mut self, identifier: &str) -> bool;
    fn get_car_status(&self, identifier: &str) -> Option<CarStatus>;
    fn get_available_cars(&self) -> Vec<String>;                      // CHANGED: Vec<String> (war bei dir im impl Vec<&str>)

    // Zusammenspiel Personen/Autos
    fn reserve_car(&mut self, person_id: &str, car_id: &str, priority: u32) -> bool;  // CHANGED: &'a str -> &str
    fn cancel_reservation(&mut self, person_id: &str, car_id: &str) -> bool;
    fn get_reservations_for_car(&self, car_id: &str) -> Vec<String>;  // CHANGED: Vec<&str> -> Vec<String>
    fn process_reservations(&mut self) -> Vec<(String, String)>;      // CHANGED: Vec<(&'a str,&'a str)> -> Vec<(String,String)>

    fn rent_car(&mut self, person_id: &str, car_id: &str) -> bool;    // CHANGED: &'a str -> &str
    fn return_car(&mut self, person_id: &str, car_id: &str, driven_km: u32) -> bool;

    fn simulate_n_days(&mut self, n: u32);
}

impl CarSharingService for CarSharing {
    // Personen
    fn register_person(&mut self, p: Person) -> bool {
        if person_exist(&self.persons, &p) == false {
            return false;
        } else {
            self.persons.push(p);
        }
        true
    }

    fn unregister_person(&mut self, identifier: &str) -> bool {
        if find_index_persons(&self.persons, identifier) == None {
            return false;
        }

        if find_persons_rentals(&self.rentals, identifier) {
            return false;
        }

        self.reservations = self.reservations
            .iter()
            .filter(|r| r.person_id != identifier)
            .cloned()
            .collect();

        self.persons = self.persons
            .iter()
            .filter(|p| p.identifier != identifier)
            .cloned()
            .collect();

        true
    }

    fn renew_license(&mut self, identifier: &str, new_valid_days: u32) -> bool {
        if let Some(index) = find_index_persons(&self.persons, identifier) {
            self.persons[index].license_valid_days = new_valid_days;
            self.persons[index].status = PersonStatus::Active;
            true
        } else {
            false
        }
    }

    fn get_person_status(&self, identifier: &str) -> Option<PersonStatus> {
        if let Some(index) = find_index_persons(&self.persons, identifier) {
            Some(self.persons[index].status.clone())
        } else {
            None
        }
    }

    // Autos
    fn register_car(&mut self, c: Car) -> bool {
        if car_exist(&self.cars, &c) == false {
            return false;
        }
        if c.mileage > 200000 {
            return false;
        }
        if retirement_score(&c) > 1.0 {
            return false;
        } else {
            self.cars.push(c);
        }
        true
    }

    fn unregister_car(&mut self, identifier: &str) -> bool {
        if let Some(_) = find_index_cars_rentals(&self.rentals, identifier) {
            return false;
        }

        let car_index = if let Some(c_index) = find_index_cars(&self.cars, identifier) {
            c_index
        } else {
            return false;
        };

        let car = &self.cars[car_index];

        // CHANGED: damit kein "move out of borrowed content" passiert
        if matches!(&car.status, CarStatus::Maintenance(_)) {
            return false;
        }
        if matches!(&car.status, CarStatus::Tuv(_)) {
            return false;
        }

        self.reservations = self.reservations
            .iter()
            .filter(|r| r.car_id != identifier)
            .cloned()
            .collect();

        self.cars = self.cars
            .iter()
            .filter(|c| c.identifier != identifier)
            .cloned()
            .collect();

        true
    }

    fn get_car_status(&self, identifier: &str) -> Option<CarStatus> {
        if let Some(index) = find_index_cars(&self.cars, identifier) {
            Some(self.cars[index].status.clone())
        } else {
            None
        }
    }

    fn get_available_cars(&self) -> Vec<String> {
        // CHANGED: Vec<&str> -> Vec<String>
        let mut av_cars: Vec<String> = Vec::new();
        for car in self.cars.iter() {
            if car.status == CarStatus::Available {
                av_cars.push(car.identifier.clone()); // CHANGED: clone String
            }
        }
        av_cars
    }

    fn reserve_car(&mut self, person_id: &str, car_id: &str, priority: u32) -> bool {
        if p_can_reserve(&self.persons, &self.rentals, &self.reservations, person_id, car_id) {
            self.reservations.push(Reservation {
                person_id: person_id.to_string(), // CHANGED: speichern als String
                car_id: car_id.to_string(),       // CHANGED
                priority,
            });
            true
        } else {
            false
        }
    }

    fn cancel_reservation(&mut self, person_id: &str, car_id: &str) -> bool {
        if let Some(index) = find_index_reservations(&self.reservations, person_id, car_id) {
            self.reservations.remove(index);
            true
        } else {
            false
        }
    }

    fn get_reservations_for_car(&self, car_id: &str) -> Vec<String> {
        // CHANGED: Vec<&str> -> Vec<String>
        persons_with_reservation_for_car(&self.reservations, car_id)
    }

    fn process_reservations(&mut self) -> Vec<(String, String)> {
        // CHANGED: Rückgabe Vec<(String,String)>
        self.reservations
            .sort_by(|r0, r1| r1.priority.cmp(&r0.priority));

        let reservations = self.reservations.clone();

        let mut processed_reservations: Vec<(String, String)> = Vec::new(); // CHANGED

        for r in reservations.iter() {
            if self.rent_car(&r.person_id, &r.car_id) {
                processed_reservations.push((r.person_id.clone(), r.car_id.clone())); // CHANGED
            }
        }

        for (person_id, _) in processed_reservations.iter() {
            while let Some(index) = find_persons_reservations(&self.reservations, person_id) {
                self.reservations.remove(index);
            }
        }

        processed_reservations
    }

    fn rent_car(&mut self, person_id: &str, car_id: &str) -> bool {
        if p_can_rent_car(&self.rentals, &self.persons, &self.cars, person_id, car_id) {
            self.rentals.push((person_id.to_string(), car_id.to_string())); // CHANGED: speichern als String

            if let Some(index) = find_index_cars(&self.cars, car_id) {
                self.cars[index].status = CarStatus::Rented;
                self.cars[index].rental_count = self.cars[index].rental_count + 1;
            }
            true
        } else {
            false
        }
    }

    fn return_car(&mut self, person_id: &str, car_id: &str, driven_km: u32) -> bool {
        let index_rental = if let Some(index_r) = find_index_rentals(&self.rentals, person_id, car_id) {
            index_r
        } else {
            return false;
        };

        let index_car = if let Some(index_c) = find_index_cars(&self.cars, car_id) {
            index_c
        } else {
            return false;
        };

        let start_km = self.cars[index_car].mileage;
        self.cars[index_car].mileage = self.cars[index_car].mileage + driven_km;

        if retirement_score(&self.cars[index_car]) > 1.0 {
            self.cars[index_car].status = CarStatus::Retired;
        } else if let Some(new_state) = check_maintenance_or_tuv(start_km, driven_km) {
            self.cars[index_car].status = new_state;
        } else {
            self.cars[index_car].status = CarStatus::Available;
        }

        self.rentals.remove(index_rental);
        true
    }

    fn simulate_n_days(&mut self, n: u32) {
        let mut days = n;

        while days > 0 {
            self.current_day = self.current_day + 1;

            for p in self.persons.iter_mut() {
                if p.license_valid_days > 0 {
                    p.license_valid_days = p.license_valid_days - 1
                }
                if p.license_valid_days == 0 {
                    p.status = PersonStatus::Blocked;
                }
            }

            for c in self.cars.iter_mut() {
                c.age_days = c.age_days + 1;
            }

            for c in self.cars.iter_mut() {
                if c.age_days == MAX_AGE_DAYS && c.status != CarStatus::Rented {
                    c.status = CarStatus::Retired
                }
                if c.rental_count == MAX_RENTALS && c.status != CarStatus::Rented {
                    c.status = CarStatus::Retired
                }
                if retirement_score(c) > 1.0 && c.status != CarStatus::Rented {
                    c.status = CarStatus::Retired
                }
            }

            // CHANGED: status.clone(), damit wir nicht aus &mut c.status "raus-moven"
            for c in self.cars.iter_mut() {
                if let CarStatus::Maintenance(days_left) = c.status.clone() {
                    if days_left > 1 {
                        c.status = CarStatus::Maintenance(days_left - 1);
                    } else {
                        c.status = CarStatus::Available;
                    }
                }
            }

            // CHANGED: status.clone() auch hier
            for c in self.cars.iter_mut() {
                if let CarStatus::Tuv(days_left) = c.status.clone() {
                    if days_left > 1 {
                        c.status = CarStatus::Tuv(days_left - 1);
                    } else {
                        c.status = CarStatus::Available;
                    }
                }
            }

            self.process_reservations();
            days = days - 1;
        }
    }
}

// -------------------- Hilfsfunktionen --------------------

fn find_index_rentals(rentals: &Vec<(String, String)>, person_id: &str, car_id: &str) -> Option<usize> {
    // CHANGED: rentals type Vec<(String,String)>
    for (index, rental) in rentals.iter().enumerate() {
        if rental.0 == person_id && rental.1 == car_id {
            return Some(index);
        }
    }
    None
}

fn find_persons_reservations(reservations: &Vec<Reservation>, person_id: &str) -> Option<usize> {
    // CHANGED: Reservation ohne Lifetime
    for (index, reservation) in reservations.iter().enumerate() {
        if reservation.person_id == person_id {
            return Some(index);
        }
    }
    None
}

fn find_persons_rentals(rentals: &Vec<(String, String)>, person_id: &str) -> bool {
    // CHANGED: rentals type
    for rental in rentals.iter() {
        if rental.0 == person_id {
            return true;
        }
    }
    false
}

fn p_can_rent_car(
    rentals: &Vec<(String, String)>, // CHANGED
    persons: &Vec<Person>,           // CHANGED
    cars: &Vec<Car>,                 // CHANGED
    person_id: &str,                 // CHANGED
    car_id: &str,                    // CHANGED
) -> bool {
    for p in rentals.iter() {
        if person_id == p.0 {
            return false
        }
    }

    for p in persons.iter() {
        if p.identifier == person_id && p.status == PersonStatus::Blocked {
            return false
        }
    }

    for c in cars.iter() {
        if c.identifier == car_id && c.status != CarStatus::Available {
            return false
        }
    }
    true
}

fn persons_with_reservation_for_car(reservations: &Vec<Reservation>, car_id: &str) -> Vec<String> {
    // CHANGED: Vec<&str> -> Vec<String>
    reservations
        .iter()
        .filter(|reservation| reservation.car_id == car_id)
        .map(|reservation| reservation.person_id.clone())
        .collect()
}

fn p_can_reserve(
    persons: &Vec<Person>,               // CHANGED
    rentals: &Vec<(String, String)>,     // CHANGED
    reservations: &Vec<Reservation>,     // CHANGED
    person_id: &str,
    car_id: &str,                        // CHANGED: &'a str -> &str
) -> bool {
    for p in persons.iter() {
        if p.identifier == person_id && p.status == PersonStatus::Blocked {
            return false;
        }
    }

    for p in rentals.iter() {
        if person_id == p.0 {
            return false;
        }
    }

    for pc in reservations.iter() {
        if person_id == pc.person_id && car_id == pc.car_id {
            return false;
        }
    }
    true
}

fn find_index_reservations(reservations: &Vec<Reservation>, person_id: &str, car_id: &str) -> Option<usize> {
    // CHANGED
    for (index, reservation) in reservations.iter().enumerate() {
        if reservation.person_id == person_id && reservation.car_id == car_id {
            return Some(index);
        }
    }
    None
}

// Cars
fn car_exist(cars: &Vec<Car>, c: &Car) -> bool {
    // CHANGED
    for car in cars.iter() {
        if c.identifier == car.identifier {
            return false;
        }
    }
    true
}

fn check_maintenance_or_tuv(mileage: u32, driven_km: u32) -> Option<CarStatus> {
    // CHANGED: lifetime entfernt
    if mileage == MAX_KM {
        Some(CarStatus::Retired)
    } else if (mileage / TUV_KM) < ((mileage + driven_km) / TUV_KM) {
        Some(CarStatus::Tuv(TUV_DAYS))
    } else if (mileage / MAINTENANCE_KM) < ((mileage + driven_km) / MAINTENANCE_KM) {
        Some(CarStatus::Maintenance(MAINTENANCE_DAYS))
    } else {
        None
    }
}

fn retirement_score(car: &Car) -> f32 {
    // CHANGED
    (car.age_days as f32 / MAX_AGE_DAYS as f32)
        + (car.mileage as f32 / MAX_KM as f32)
        + (car.rental_count as f32 / MAX_RENTALS as f32)
}

fn find_index_cars_rentals(rentals: &Vec<(String, String)>, car_id: &str) -> Option<usize> {
    // CHANGED
    for (index, rental) in rentals.iter().enumerate() {
        if rental.1 == car_id {
            return Some(index);
        }
    }
    None
}

fn find_index_cars(cars: &Vec<Car>, identifier: &str) -> Option<usize> {
    // CHANGED
    for (index, car) in cars.iter().enumerate() {
        if identifier == car.identifier {
            return Some(index);
        }
    }
    None
}

// Personen
fn person_exist(persons: &Vec<Person>, p: &Person) -> bool {
    // CHANGED
    for person in persons.iter() {
        if p.identifier == person.identifier {
            return false;
        }
    }
    true
}

fn find_index_persons(persons: &Vec<Person>, identifier: &str) -> Option<usize> {
    // CHANGED
    for (index, person) in persons.iter().enumerate() {
        if identifier == person.identifier {
            return Some(index);
        }
    }
    None
}

impl CarSharing {
    // CHANGED: convenience constructor für UI
    pub fn new() -> Self {
        Self {
            persons: vec![],
            cars: vec![],
            rentals: vec![],
            reservations: vec![],
            current_day: 0,
        }
    }
}