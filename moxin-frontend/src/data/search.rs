use anyhow::{anyhow, Result};
use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::data::*;
use moxin_protocol::protocol::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub enum SearchAction {
    Results(Vec<Model>),
    Error,
}

pub enum SearchCommand {
    Search(String),
    LoadFeaturedModels,
}

#[derive(Default)]
pub enum SearchState {
    #[default]
    Idle,
    Pending,
    Errored,
}

pub struct Search {
    pub keyword: Option<String>,
    pub current_command: Option<SearchCommand>,
    pub next_command: Option<SearchCommand>,
    pub sender: Sender<SearchAction>,
    pub receiver: Receiver<SearchAction>,
    pub state: SearchState,
}

impl Default for Search {
    fn default() -> Self {
        Search::new()
    }
}

impl Search {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let search = Self {
            keyword: None,
            current_command: None,
            next_command: None,
            sender: tx,
            receiver: rx,
            state: SearchState::Idle,
        };
        search
    }

    pub fn load_featured_models(&mut self, backend: &Backend) {
        match self.state {
            SearchState::Pending => {
                self.next_command = Some(SearchCommand::LoadFeaturedModels);
                return;
            }
            SearchState::Idle | SearchState::Errored => {
                self.state = SearchState::Pending;
                self.keyword = None;
                self.next_command = None;
            }
        }

        let (tx, rx) = channel();

        let store_search_tx = self.sender.clone();
        backend
            .command_sender
            .send(Command::GetFeaturedModels(tx))
            .unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(models) => {
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error fetching models: {:?}", err);
                        store_search_tx.send(SearchAction::Error).unwrap();
                    }
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

    pub fn run_or_enqueue(&mut self, keyword: String, backend: &Backend) {
        match self.state {
            SearchState::Pending => {
                self.next_command = Some(SearchCommand::Search(keyword));
                return;
            }
            SearchState::Idle | SearchState::Errored => {
                self.state = SearchState::Pending;
                self.current_command = Some(SearchCommand::Search(keyword.clone()));
                self.next_command = None;
            }
        }

        let (tx, rx) = channel();

        let store_search_tx = self.sender.clone();
        backend
            .command_sender
            .send(Command::SearchModels(keyword.clone(), tx))
            .unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(models) => {
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error fetching models: {:?}", err);
                        store_search_tx.send(SearchAction::Error).unwrap();
                    }
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

    pub fn process_results(&mut self, backend: &Backend) -> Result<Vec<Model>> {
        for msg in self.receiver.try_iter() {
            match msg {
                SearchAction::Results(models) => {
                    self.state = SearchState::Idle;
                    if let Some(SearchCommand::Search(keyword)) = self.current_command.take() {
                        self.keyword = Some(keyword);
                    }
                    match self.next_command.take() {
                        Some(SearchCommand::Search(next_keyword)) => {
                            self.run_or_enqueue(next_keyword, backend);
                        }
                        Some(SearchCommand::LoadFeaturedModels) => {
                            self.load_featured_models(backend);
                        }
                        None => {}
                    }
                    return Ok(models);
                }
                SearchAction::Error => {
                    self.state = SearchState::Errored;
                    return Err(anyhow!("Error fetching models from the server"));
                }
            }
        }
        Err(anyhow!("Unkown error fetching models from the server"))
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.state, SearchState::Pending)
    }

    pub fn was_error(&self) -> bool {
        matches!(self.state, SearchState::Errored)
    }
}
