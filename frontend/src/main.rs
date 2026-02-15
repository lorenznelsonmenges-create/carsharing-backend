use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

use rust_frontend::carsharing::{
    Car, CarSharing, CarSharingService, CarStatus, Person, PersonStatus,
};

#[derive(Clone, PartialEq)]
enum Tab {
    Persons,
    Cars,
    Reservations,
    Rentals,
    Simulation,
}

fn tab_button(current: &Tab, tab: Tab, label: &str, on_click: Callback<MouseEvent>) -> Html {
    let is_active = *current == tab;
    let style = if is_active {
        "padding:8px 12px; border:1px solid #333; background:#333; color:#fff; border-radius:10px; cursor:pointer;"
    } else {
        "padding:8px 12px; border:1px solid #333; background:#fff; color:#333; border-radius:10px; cursor:pointer;"
    };

    html! { <button style={style} onclick={on_click}>{label}</button> }
}

#[function_component(App)]
fn app() -> Html {
    let tab = use_state(|| Tab::Persons);
    let cs = use_state(CarSharing::new);

    // --- LADE-MECHANISMUS ---
    {
        let cs = cs.clone();
        use_effect(move || {
            spawn_local(async move {
                let fetched_cs: CarSharing = Request::get("/api/state")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                cs.set(fetched_cs);
            });
            || ()
        });
    }

    // --- NEUER SPEICHER-MECHANISMUS ---
    let save_state = {
        let info = use_state(|| String::new());
        Callback::from(move |model: CarSharing| {
            let info = info.clone();
            spawn_local(async move {
                let request = Request::post("/api/state")
                    .json(&model);
                
                match request {
                    Ok(req) => {
                        if let Err(_) = req.send().await {
                            info.set("Fehler: Konnte Zustand nicht ans Backend senden.".to_string());
                        }
                    }
                    Err(_) => {
                        info.set("Fehler: Interner Fehler beim Erstellen der Anfrage.".to_string());
                    }
                }
            });
        })
    };

    let info = use_state(|| String::new());

    // ---------- Form States ----------
    let p_id = use_state(|| "".to_string());
    let p_days = use_state(|| "".to_string());
    let c_id = use_state(|| "".to_string());
    let c_km = use_state(|| "".to_string());
    let c_age = use_state(|| "".to_string());
    let r_person = use_state(|| "".to_string());
    let r_car = use_state(|| "".to_string());
    let r_prio = use_state(|| "1".to_string());
    let ret_person = use_state(|| "".to_string());
    let ret_car = use_state(|| "".to_string());
    let ret_km = use_state(|| "".to_string());
    let sim_days = use_state(|| "".to_string());

    let on_reset = {
        let cs = cs.clone();
        let info = info.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let new_model = CarSharing::new();
            save_state.emit(new_model.clone()); // Send reset state to backend
            cs.set(new_model);
            info.set("State an Backend gesendet und zurückgesetzt.".to_string());
        })
    };

    // ========== Tab Switch Callbacks ==========
    let set_tab_persons = { let tab = tab.clone(); Callback::from(move |_| tab.set(Tab::Persons)) };
    let set_tab_cars = { let tab = tab.clone(); Callback::from(move |_| tab.set(Tab::Cars)) };
    let set_tab_res = { let tab = tab.clone(); Callback::from(move |_| tab.set(Tab::Reservations)) };
    let set_tab_rentals = { let tab = tab.clone(); Callback::from(move |_| tab.set(Tab::Rentals)) };
    let set_tab_sim = { let tab = tab.clone(); Callback::from(move |_| tab.set(Tab::Simulation)) };

    // ========== Persons Actions ==========
    let on_add_person = {
        let cs = cs.clone();
        let info = info.clone();
        let p_id = p_id.clone();
        let p_days = p_days.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let id = (*p_id).trim().to_string();
            if id.is_empty() { info.set("Person-ID darf nicht leer sein.".to_string()); return; }
            let days = match (*p_days).trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("license_valid_days muss eine Zahl sein.".to_string()); return; }
            };
            let ok = model.register_person(Person { identifier: id.clone(), license_valid_days: days, status: PersonStatus::Active });
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Person '{}' angelegt.", id));
            } else {
                info.set("Person existiert schon oder konnte nicht angelegt werden.".to_string());
            }
        })
    };

    let on_remove_person = {
        let cs = cs.clone();
        let info = info.clone();
        let p_id = p_id.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let id = (*p_id).trim().to_string();
            if id.is_empty() { info.set("Zum Entfernen bitte Person-ID eingeben.".to_string()); return; }
            let ok = model.unregister_person(&id);
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Person '{}' entfernt.", id));
            } else {
                info.set("Person konnte nicht entfernt werden.".to_string());
            }
        })
    };

    let on_renew_license = {
        let cs = cs.clone();
        let info = info.clone();
        let p_id = p_id.clone();
        let p_days = p_days.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let id = (*p_id).trim().to_string();
            if id.is_empty() { info.set("Bitte Person-ID eingeben.".to_string()); return; }
            let days = match (*p_days).trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("new_valid_days muss eine Zahl sein.".to_string()); return; }
            };
            let ok = model.renew_license(&id, days);
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Führerschein für '{}' erneuert.", id));
            } else {
                info.set("Person nicht gefunden.".to_string());
            }
        })
    };

    // ========== Cars Actions ==========
    let on_add_car = {
        let cs = cs.clone();
        let info = info.clone();
        let c_id = c_id.clone();
        let c_km = c_km.clone();
        let c_age = c_age.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let id = (*c_id).trim().to_string();
            if id.is_empty() { info.set("Car-ID darf nicht leer sein.".to_string()); return; }
            let mileage = match (*c_km).trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("mileage muss eine Zahl sein.".to_string()); return; }
            };
            let age_days = match (*c_age).trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("age_days muss eine Zahl sein.".to_string()); return; }
            };
            let ok = model.register_car(Car { identifier: id.clone(), mileage, status: CarStatus::Available, age_days, rental_count: 0 });
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Auto '{}' angelegt.", id));
            } else {
                info.set("Auto konnte nicht angelegt werden.".to_string());
            }
        })
    };

    let on_remove_car = {
        let cs = cs.clone();
        let info = info.clone();
        let c_id = c_id.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let id = (*c_id).trim().to_string();
            if id.is_empty() { info.set("Zum Entfernen bitte Car-ID eingeben.".to_string()); return; }
            let ok = model.unregister_car(&id);
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Auto '{}' entfernt.", id));
            } else {
                info.set("Auto konnte nicht entfernt werden.".to_string());
            }
        })
    };

    // ========== Reservation Actions ==========
    let on_reserve = {
        let cs = cs.clone();
        let info = info.clone();
        let r_person = r_person.clone();
        let r_car = r_car.clone();
        let r_prio = r_prio.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let person_id = (*r_person).trim().to_string();
            let car_id = (*r_car).trim().to_string();
            if person_id.is_empty() || car_id.is_empty() { info.set("Bitte Person-ID und Car-ID eingeben.".to_string()); return; }
            let prio = match (*r_prio).trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("priority muss eine Zahl sein.".to_string()); return; }
            };
            let ok = model.reserve_car(&person_id, &car_id, prio);
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Reservierung gesetzt: {} -> {}.", person_id, car_id));
            } else {
                info.set("Reservierung nicht möglich.".to_string());
            }
        })
    };

    let on_cancel_reservation = {
        let cs = cs.clone();
        let info = info.clone();
        let r_person = r_person.clone();
        let r_car = r_car.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let person_id = (*r_person).trim().to_string();
            let car_id = (*r_car).trim().to_string();
            if person_id.is_empty() || car_id.is_empty() { info.set("Bitte Person-ID und Car-ID eingeben.".to_string()); return; }
            let ok = model.cancel_reservation(&person_id, &car_id);
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Reservierung storniert: {} -> {}.", person_id, car_id));
            } else {
                info.set("Reservierung nicht gefunden.".to_string());
            }
        })
    };

    let on_process_reservations = {
        let cs = cs.clone();
        let info = info.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let processed = model.process_reservations();
            save_state.emit(model.clone());
            cs.set(model);
            if processed.is_empty() {
                info.set("Keine Reservierungen verarbeitet.".to_string());
            } else {
                info.set(format!("Verarbeitet: {:?}", processed));
            }
        })
    };

    // ========== Return Action ==========
    let on_return = {
        let cs = cs.clone();
        let info = info.clone();
        let ret_person = ret_person.clone();
        let ret_car = ret_car.clone();
        let ret_km = ret_km.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let person_id = (*ret_person).trim().to_string();
            let car_id = (*ret_car).trim().to_string();
            if person_id.is_empty() || car_id.is_empty() { info.set("Bitte Person-ID und Car-ID eingeben.".to_string()); return; }
            let driven_km = match (*ret_km).trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("driven_km muss eine Zahl sein.".to_string()); return; }
            };
            let ok = model.return_car(&person_id, &car_id, driven_km);
            if ok {
                save_state.emit(model.clone());
                cs.set(model);
                info.set(format!("Auto zurückgegeben: {} -> {}.", person_id, car_id));
            } else {
                info.set("Return fehlgeschlagen.".to_string());
            }
        })
    };

    // ========== Simulation ==========
    let on_simulate = {
        let cs = cs.clone();
        let info = info.clone();
        let sim_days = sim_days.clone();
        let save_state = save_state.clone();
        Callback::from(move |_| {
            let mut model = (*cs).clone();
            let n = match sim_days.trim().parse::<u32>() {
                Ok(v) => v,
                Err(_) => { info.set("Simulationstage müssen eine Zahl sein.".to_string()); return; }
            };
            model.simulate_n_days(n);
            save_state.emit(model.clone());
            cs.set(model);
            info.set(format!("Simulation durchgeführt: {} Tage.", n));
        })
    };

    // ========== Inputs: oninput callbacks ==========
    let on_p_id = { let p_id = p_id.clone(); Callback::from(move |e: InputEvent| { p_id.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_p_days = { let p_days = p_days.clone(); Callback::from(move |e: InputEvent| { p_days.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_c_id = { let c_id = c_id.clone(); Callback::from(move |e: InputEvent| { c_id.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_c_km = { let c_km = c_km.clone(); Callback::from(move |e: InputEvent| { c_km.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_c_age = { let c_age = c_age.clone(); Callback::from(move |e: InputEvent| { c_age.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_r_person = { let r_person = r_person.clone(); Callback::from(move |e: InputEvent| { r_person.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_r_car = { let r_car = r_car.clone(); Callback::from(move |e: InputEvent| { r_car.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_r_prio = { let r_prio = r_prio.clone(); Callback::from(move |e: InputEvent| { r_prio.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_ret_person = { let ret_person = ret_person.clone(); Callback::from(move |e: InputEvent| { ret_person.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_ret_car = { let ret_car = ret_car.clone(); Callback::from(move |e: InputEvent| { ret_car.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_ret_km = { let ret_km = ret_km.clone(); Callback::from(move |e: InputEvent| { ret_km.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };
    let on_sim_days = { let sim_days = sim_days.clone(); Callback::from(move |e: InputEvent| { sim_days.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()); }) };

    // ========== Render current tab ==========
    let current_tab = (*tab).clone();
    let model = (*cs).clone();
    let panel_style = "border:1px solid #ddd; border-radius:16px; padding:16px; margin-top:12px;";
    let row_style = "display:flex; gap:8px; flex-wrap:wrap; align-items:center; margin:8px 0;";
    let input_style = "padding:8px 10px; border:1px solid #ccc; border-radius:10px; min-width:220px;";
    let button_style = "padding:8px 12px; border:1px solid #333; background:#fff; border-radius:10px; cursor:pointer;";
    let small = "color:#555; font-size: 14px;";

    let content = match current_tab {
        Tab::Persons => html! {
            <section style={panel_style}>
                <h2>{"Persons"}</h2>
                <div style={row_style}>
                    <input style={input_style} placeholder="Person-ID" value={(*p_id).clone()} oninput={on_p_id}/>
                    <input style={input_style} placeholder="license_valid_days" value={(*p_days).clone()} oninput={on_p_days}/>
                </div>
                <div style={row_style}>
                    <button style={button_style} onclick={on_add_person}>{"Add Person"}</button>
                    <button style={button_style} onclick={on_remove_person}>{"Remove Person (by ID)"}</button>
                    <button style={button_style} onclick={on_renew_license}>{"Renew License (ID + days)"}</button>
                </div>
                <p style={small}>{format!("Persons: {}", model.persons.len())}</p>
                <ul>{ for model.persons.iter().map(|p| html!{ <li>{format!("{} | days:{} | status:{:?}", p.identifier, p.license_valid_days, p.status)}</li> }) }</ul>
            </section>
        },
        Tab::Cars => html! {
            <section style={panel_style}>
                <h2>{"Cars"}</h2>
                <div style={row_style}>
                    <input style={input_style} placeholder="Car-ID" value={(*c_id).clone()} oninput={on_c_id}/>
                    <input style={input_style} placeholder="mileage" value={(*c_km).clone()} oninput={on_c_km}/>
                    <input style={input_style} placeholder="age_days" value={(*c_age).clone()} oninput={on_c_age}/>
                </div>
                <div style={row_style}>
                    <button style={button_style} onclick={on_add_car}>{"Add Car"}</button>
                    <button style={button_style} onclick={on_remove_car}>{"Remove Car (by ID)"}</button>
                </div>
                <p style={small}>{format!("Cars: {}", model.cars.len())}</p>
                <p style={small}>{format!("Available: {:?}", model.get_available_cars())}</p>
                <ul>{ for model.cars.iter().map(|c| html!{ <li>{format!("{} | km:{} | age:{} | rentals:{} | status:{:?}", c.identifier, c.mileage, c.age_days, c.rental_count, c.status)}</li> }) }</ul>
            </section>
        },
        Tab::Reservations => html! {
            <section style={panel_style}>
                <h2>{"Reservations"}</h2>
                <div style={row_style}>
                    <input style={input_style} placeholder="Person-ID" value={(*r_person).clone()} oninput={on_r_person}/>
                    <input style={input_style} placeholder="Car-ID" value={(*r_car).clone()} oninput={on_r_car}/>
                    <input style={input_style} placeholder="priority" value={(*r_prio).clone()} oninput={on_r_prio}/>
                </div>
                <div style={row_style}>
                    <button style={button_style} onclick={on_reserve}>{"Reserve"}</button>
                    <button style={button_style} onclick={on_cancel_reservation}>{"Cancel Reservation"}</button>
                    <button style={button_style} onclick={on_process_reservations}>{"Process Reservations"}</button>
                </div>
                <p style={small}>{format!("Reservations: {}", model.reservations.len())}</p>
                <ul>{ for model.reservations.iter().map(|r| html!{ <li>{format!("{} -> {} (prio {})", r.person_id, r.car_id, r.priority)}</li> }) }</ul>
            </section>
        },
        Tab::Rentals => html! {
            <section style={panel_style}>
                <h2>{"Active Rentals"}</h2>
                <h3 style="margin-top:14px;">{"Return"}</h3>
                <div style={row_style}>
                    <input style={input_style} placeholder="Person-ID" value={(*ret_person).clone()} oninput={on_ret_person}/>
                    <input style={input_style} placeholder="Car-ID" value={(*ret_car).clone()} oninput={on_ret_car}/>
                    <input style={input_style} placeholder="driven_km" value={(*ret_km).clone()} oninput={on_ret_km}/>
                    <button style={button_style} onclick={on_return}>{"Return Car"}</button>
                </div>
                <p style={small}>{format!("Rentals: {}", model.rentals.len())}</p>
                <ul>{ for model.rentals.iter().map(|(p,c)| html!{ <li>{format!("{} -> {}", p, c)}</li> }) }</ul>
            </section>
        },
        Tab::Simulation => html! {
            <section style={panel_style}>
                <h2>{"Simulation"}</h2>
                <div style={row_style}>
                    <input style={input_style} placeholder="n days" value={(*sim_days).clone()} oninput={on_sim_days}/>
                    <button style={button_style} onclick={on_simulate}>{"Simulate n days"}</button>
                    <button style={button_style} onclick={on_reset}>{"Reset state"}</button>
                </div>
                <p style={small}>{format!("Current day: {}", model.current_day)}</p>
                <p style={small}>{"Hinweis: simulate_n_days() verarbeitet am Ende jedes Tages process_reservations()."}</p>
            </section>
        },
    };

    html! {
        <main style="font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif; padding: 16px; max-width: 980px; margin: 0 auto;">
            <h1>{"Carsharing Frontend"}</h1>
            <div style="display:flex; gap:10px; flex-wrap:wrap;">
                { tab_button(&tab, Tab::Persons, "Persons", set_tab_persons) }
                { tab_button(&tab, Tab::Cars, "Cars", set_tab_cars) }
                { tab_button(&tab, Tab::Reservations, "Reservations", set_tab_res) }
                { tab_button(&tab, Tab::Rentals, "Active Rentals", set_tab_rentals) }
                { tab_button(&tab, Tab::Simulation, "Simulation", set_tab_sim) }
            </div>
            <p style="margin-top:12px; padding:10px 12px; border:1px solid #eee; border-radius:12px; background:#fafafa;">
                <strong>{"Status: "}</strong>{(*info).clone()}
            </p>
            {content}
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}