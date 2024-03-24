use std::collections::HashMap;

use crate::{message::Message, HandleMessageError};

pub fn create_entry_id_map(
    messages: &[Message],
) -> Result<HashMap<String, &Message>, HandleMessageError> {
    messages.iter().try_fold(
        HashMap::new(),
        |mut acc, m| -> Result<_, HandleMessageError> {
            let entry_id = m.entry_id()?;
            acc.insert(entry_id, m);
            Ok(acc)
        },
    )
}
