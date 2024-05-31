use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    fmt::{self, Display, Formatter},
    os::linux::raw::stat,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;

use crate::token::Token;

struct IDAllocatorInner {
    next_id: usize,
}

struct IDAllocator {
    inner: Mutex<IDAllocatorInner>,
}

impl IDAllocator {
    pub fn next(&self) -> usize {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        id
    }
}
lazy_static! {
    static ref ID_ALLOCATOR: IDAllocator = IDAllocator {
        inner: Mutex::new(IDAllocatorInner { next_id: 0 }),
    };
}

#[derive(PartialEq)]
enum Symbol {
    LeftParen,
    Or,
}

#[derive(PartialEq, Eq, Hash, Debug)]
enum Transition {
    Epsilon,
    Char(char),
}

#[derive(Debug, PartialEq, Eq)]
struct StateInner {
    /// The transitions of the state
    transitions: Vec<(Transition, Arc<State>)>,
    /// Whether the state is accepting
    accepting: bool,
}

impl Display for StateInner {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "accepting: {} ", self.accepting)?;
        for (trans, to) in self.transitions.iter() {
            write!(f, "[{:?} -> {} ] ", trans, to.id)?;
        }
        Ok(())
    }
}

impl StateInner {
    pub fn new(accepting: bool) -> Self {
        Self {
            transitions: Vec::new(),
            accepting,
        }
    }
    pub fn add_transition(&mut self, trans: Transition, to: Arc<State>) {
        self.transitions.push((trans, to));
    }
}

#[derive(Debug)]
struct State {
    id: usize,
    inner: RefCell<StateInner>,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl State {
    pub fn new(accepting: bool) -> Self {
        Self {
            id: ID_ALLOCATOR.next(),
            inner: RefCell::new(StateInner::new(accepting)),
        }
    }
}

#[derive(Debug)]
pub struct NFA {
    /// The states of the NFA
    states: Vec<Arc<State>>,
    /// The initial state of the NFA
    initial: Arc<State>,
}

impl Display for NFA {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"-------------------------------------\n")?;
        write!(f, "Initial: {}\n", self.initial.id)?;
        for state in self.states.iter() {
            write!(f, "{} {}\n", state.id, state.inner.borrow())?;
        }
        write!(f,"-------------------------------------\n")?;
        Ok(())
    }
}

impl NFA {
    fn new() -> Self {
        let state = Arc::new(State::new(false));
        Self {
            states: vec![state.clone()],
            initial: state.clone(),
        }
    }

    fn add_state(&mut self, accepting: bool) -> Arc<State> {
        let state = Arc::new(State::new(accepting));
        self.states.push(state.clone());
        state
    }

    // connect a nfa to other nfa
    fn connect_other(&mut self, other: NFA) {
        // self.accepting -> other.initial
        self.states
            .iter_mut()
            .filter(|state| state.inner.borrow().accepting)
            .for_each(|state| {
                let mut inner = state.inner.borrow_mut();
                inner.accepting = false;
                inner.add_transition(Transition::Epsilon, Arc::clone(&other.initial));
            });
        self.states.extend(other.states);
    }

    fn merge_other(&mut self, other: NFA) {
        let new_initial = self.add_state(false);
        new_initial
            .inner
            .borrow_mut()
            .add_transition(Transition::Epsilon, self.initial.clone());
        new_initial
            .inner
            .borrow_mut()
            .add_transition(Transition::Epsilon, other.initial.clone());
        self.initial = new_initial;
        self.states.extend(other.states);
    }

    #[allow(dead_code)]
    pub fn new_from_token(token: &Token) -> Self {
        let token = format!("({})", &token.value);
        let mut symbol_stack: Vec<Symbol> = Vec::new();
        let mut nfa_stack: Vec<NFA> = Vec::new();
        let handle_symbol = |symbol_stack: &mut Vec<Symbol>, nfa_stack: &mut Vec<NFA>| {
            let symbol = symbol_stack.pop().unwrap();
            let mut nfa1 = nfa_stack.pop().unwrap();
            let nfa2 = nfa_stack.pop().unwrap();
            match symbol {
                Symbol::Or => {
                    // nfa1 | nfa2
                    // nfa -> nfa1
                    //     -> nfa2
                    nfa1.merge_other(nfa2);
                    nfa_stack.push(nfa1);
                }
                _ => {}
            }
        };
        for (idx, c) in token.chars().enumerate() {
            match c {
                '(' => {
                    symbol_stack.push(Symbol::LeftParen);
                    nfa_stack.push(NFA::new());
                }
                ')' => {
                    // while the top of the symbol stack is not '('
                    while *symbol_stack.last().unwrap() != Symbol::LeftParen {
                        handle_symbol(&mut symbol_stack, &mut nfa_stack);
                    }
                    symbol_stack.pop();
                }
                '|' => {
                    symbol_stack.push(Symbol::Or);
                    nfa_stack.push(NFA::new());
                }
                _ => {
                    let nfa = nfa_stack.last_mut().unwrap();
                    // last -epsilon-> state1 -c-> state2
                    let last_state = nfa.states.last_mut().unwrap().clone();
                    let state1 = nfa.add_state(false);
                    // if next char is ) or |, state2 is accepting
                    let next_c = token.chars().nth(idx + 1).unwrap();
                    let state2 = nfa.add_state(next_c == '|' || next_c == ')');

                    last_state
                        .inner
                        .borrow_mut()
                        .add_transition(Transition::Epsilon, state1.clone());
                    state1
                        .inner
                        .borrow_mut()
                        .add_transition(Transition::Char(c), state2.clone());
                }
            }
        }
        nfa_stack.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;

    #[test]
    fn test_nfa() {
        let nfa = NFA::new_from_token(&Token {
            value: "a|b".to_string(),
            kind: "char".to_string(),
        });
        for state in nfa.states.iter() {
            print!("{}", state.inner.borrow());
        }
    }
}
