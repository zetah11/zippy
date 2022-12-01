use common::mir;

use super::{Frame, Interpreter, StateAction, Value};

impl Interpreter<'_> {
    /// Attempt to produce a [`Value`] from the given value. If the given value is an undefined, top-level name,
    /// this will return `None` and push that name to the worklist.
    pub fn make_value(&mut self, value: &mir::Value) -> Option<Value> {
        match &value.node {
            mir::ValueNode::Int(i) => Some(Value::Int(*i)),
            mir::ValueNode::Invalid => Some(Value::Invalid),

            mir::ValueNode::Name(name) if self.has_top_level(name) => {
                if let Some(value) = self.current().and_then(|state| state.get(name)) {
                    Some(value.clone())
                } else if self.functions.contains_key(name) {
                    Some(Value::Function(*name))
                } else {
                    self.worklist.push(
                        self.current()
                            .unwrap()
                            .split(StateAction::StoreGlobal(*name)),
                    );
                    let place = self.place_of_top_level(name)?;
                    self.current_mut().unwrap().enter(Frame::new(place));

                    None
                }
            }

            mir::ValueNode::Name(name) => {
                Some(self.current().and_then(|state| state.get(name))?.clone())
            }
        }
    }
}
