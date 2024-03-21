use std::collections::HashMap;

use crate::{message::RawMessage, HandleMessageError};

pub fn create_entry_id_map(
    messages: &[RawMessage],
) -> Result<HashMap<String, &RawMessage>, HandleMessageError> {
    messages.iter().try_fold(
        HashMap::new(),
        |mut acc, m| -> Result<_, HandleMessageError> {
            let entry_id = m.generate_record_id()?;
            acc.insert(entry_id, m);
            Ok(acc)
        },
    )
}
