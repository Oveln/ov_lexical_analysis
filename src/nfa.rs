use std::{collections::HashMap, sync::Arc};

use crate::token::Token;

struct State {
    /// The transitions of the state
    transitions: HashMap<char, Arc<State>>,
    /// Whether the state is accepting
    accepting: bool,
}

impl State {
    pub fn add_transition(&mut self, c: char, to: Arc<State>) {
        self.transitions.insert(c, to);
    }
}

struct NFA {
    /// The states of the NFA
    states: Vec<Arc<State>>,
    /// The initial state of the NFA
    initial: Arc<State>,
    /// The accepting states of the NFA
    accepting: Vec<Arc<State>>,
}

impl NFA {
    fn add_state(&mut self, accepting: bool) -> usize {
        let index = self.states.len();
        self.states.push(Arc::new(State {
            transitions: HashMap::new(),
            accepting,
        }));
        index
    }

    pub fn new_from_token(token: &Token) -> Self {
        let token = format!("({})", &token.value).chars();
        let states = vec![Arc::new(State {
            transitions: HashMap::new(),
            accepting: false,
        })];
        let initial = Arc::clone(&states[0]);
        let nfa = NFA {
            states,
            initial,
            accepting: Vec::new(),
        };
        let mut c_stack: Vec<usize> = Vec::new();
        nfa
    }
}
