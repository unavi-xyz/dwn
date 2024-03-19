use std::collections::HashMap;

use crate::{handlers::HandlerError, message::Message};

pub fn create_entry_id_map(
    messages: &[Message],
) -> Result<HashMap<String, &Message>, HandlerError> {
    messages
        .iter()
        .try_fold(HashMap::new(), |mut acc, m| -> Result<_, HandlerError> {
            let entry_id = m.generate_record_id()?;
            acc.insert(entry_id, m);
            Ok(acc)
        })
}
