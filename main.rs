use iced::widget::{button, column, text, Column};

mod api;
use api::{user_teams};


#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Join,
}

async fn my_async() {
     println!("Running !");
}


impl Counter {
    fn view(&self) -> Column<Message> {
        let teams = column![];
        teams.push(button("Join").on_press(Message::Join))
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Join => {
                // Add your decrement logic here, or handle accordingly
                println!("Join button pressed");

                let mut tasks = Vec::new();
                tasks.push(Task::perform(
                    async move {
                        my_async().await;

                    },

                ));
                Task::batch(tasks)
            }
        }
    }
}

pub fn main() -> iced::Result {
    iced::run("A cool counter", Counter::update, Counter::view)
}
