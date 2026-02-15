struct Person {
    age: u8, 
    name: [char; 10],
}

fn avg_age(persons: [Person; 10]) -> f32 {
    (persons[0].age as f32 + persons[1].age as f32 + persons[2].age as f32 + persons[3].age as f32 + persons[4].age as f32 + persons[5].age as f32 + persons[6].age as f32 + persons[7].age as f32 + persons[8].age as f32 + persons[9].age as f32) / 10.0 
}

fn main() {
    println!("Durschnittsalter: {}", avg_age(
        [
            Person {age: 10, name: ['M','i','r','i','_','_','_','_','_','_']},
            Person {age: 20, name: ['L','o','r','e','n','z','_','_','_','_']},
            Person {age: 30, name: ['M','a','s','c','h','a','_','_','_','_']},
            Person {age: 40, name: ['N','i','c','k','_','_','_','_','_','_']},
            Person {age: 50, name: ['F','l','o','_','_','_','_','_','_','_']},
            Person {age: 60, name: ['A','n','t','h','o','n','y','_','_','_']},
            Person {age: 70, name: ['D','a','n','i','_','_','_','_','_','_']},
            Person {age: 80, name: ['J','o','e','l','_','_','_','_','_','_']},
            Person {age: 90, name: ['M','a','x','_','_','_','_','_','_','_']},
            Person {age: 100, name: ['M','i','c','h','a','_','_','_','_','_']},
        ]
    )
)
} 





