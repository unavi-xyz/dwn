use dwn_core::{
    message::data::Data,
    store::{DataStore, StoreError},
};
use xdid::core::did::Did;

use crate::{
    data::{CidData, RefCount},
    NativeDbStore,
};

impl DataStore for NativeDbStore<'_> {
    fn read(&self, target: &Did, cid: &str) -> Result<Option<Data>, StoreError> {
        let tx = self
            .0
            .r_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let res = tx
            .get()
            .primary::<CidData>((target.to_string(), cid.to_string()))
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        Ok(res.and_then(|d| d.data))
    }

    fn add_ref(&self, target: &Did, cid: &str, data: Option<Data>) -> Result<(), StoreError> {
        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let key = (target.to_string(), cid.to_string());

        match tx
            .get()
            .primary::<RefCount>(key.clone())
            .map_err(|e| StoreError::BackendError(e.to_string()))?
        {
            Some(data_ref) => {
                // Update data, if provided.
                if let Some(data) = data {
                    tx.upsert(CidData {
                        key: key.clone(),
                        data: Some(data),
                    })
                    .map_err(|e| StoreError::BackendError(e.to_string()))?;
                }

                // Update ref count.
                let mut new_data_ref = data_ref.clone();
                new_data_ref.count += 1;

                tx.update(data_ref, new_data_ref)
                    .map_err(|e| StoreError::BackendError(e.to_string()))?;
            }
            None => {
                // Insert data,
                tx.insert(CidData {
                    key: key.clone(),
                    data,
                })
                .map_err(|e| StoreError::BackendError(e.to_string()))?;

                // Insert ref count,
                tx.insert(RefCount { key, count: 1 })
                    .map_err(|e| StoreError::BackendError(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        Ok(())
    }

    fn remove_ref(&self, target: &Did, cid: &str) -> Result<(), StoreError> {
        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let key = (target.to_string(), cid.to_string());

        let Some(found) = tx
            .get()
            .primary::<RefCount>(key.clone())
            .map_err(|e| StoreError::BackendError(e.to_string()))?
        else {
            return Ok(());
        };

        if found.count == 1 {
            // Remove ref count and data.
            tx.remove(found)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;

            if let Some(found_data) = tx
                .get()
                .primary::<CidData>(key.clone())
                .map_err(|e| StoreError::BackendError(e.to_string()))?
            {
                tx.remove(found_data)
                    .map_err(|e| StoreError::BackendError(e.to_string()))?;
            }
        } else {
            // Decrement ref count.
            let mut new_found = found.clone();
            new_found.count -= 1;

            tx.update(found, new_found)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use xdid::core::did::{MethodId, MethodName};

    use super::*;

    #[test]
    fn test_data_store_cleanup() {
        let ds = NativeDbStore::new_in_memory().unwrap();

        let target = Did {
            method_name: MethodName("test".to_string()),
            method_id: MethodId("test".to_string()),
        };
        let cid = &"test cid";
        let key = (target.to_string(), cid.to_string());

        ds.add_ref(&target, cid, None).unwrap();

        let tx = ds.0.r_transaction().unwrap();
        assert!(tx.get().primary::<RefCount>(key.clone()).unwrap().is_some());
        assert!(tx.get().primary::<CidData>(key.clone()).unwrap().is_some());

        ds.remove_ref(&target, cid).unwrap();

        let tx = ds.0.r_transaction().unwrap();
        assert!(tx.get().primary::<RefCount>(key.clone()).unwrap().is_none());
        assert!(tx.get().primary::<CidData>(key).unwrap().is_none());
    }
}
