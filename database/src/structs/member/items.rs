use super::{Error, Member};
use crate::{
    models::{ItemType, Quality},
    structs::{Item, ItemAmount},
};
use std::collections::hash_map::Entry;

impl Member {
    pub fn get_item(&self, item_name: String) -> Result<ItemAmount, Error> {
        let item = match self
            .get_inventory()
            .iter()
            .find(|item| item.info.name == item_name)
        {
            Some(item) => item.clone(),
            None => return Err("item not found".into()),
        };

        Ok(item)
    }

    pub async fn add_item<'a, E>(
        &mut self,
        executor: E,
        item: Item,
        amount: i32,
    ) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "INSERT INTO normal_inventory (item, member, amount)
            VALUES ($1, $2, $3)
            ON CONFLICT (item, member) DO UPDATE
            SET amount = normal_inventory.amount + $3",
            item.id,
            self.id,
            amount
        )
        .execute(executor)
        .await?;

        match self.inventory.entry(item.id.unwrap()) {
            Entry::Occupied(mut entry) => {
                let current_item = entry.get_mut();
                current_item.amount += amount;
            }
            Entry::Vacant(entry) => {
                let mut new_item = ItemAmount { info: item, amount };
                new_item.amount = amount;
                entry.insert(new_item);
            }
        }

        Ok(())
    }

    pub async fn remove_item<'a, E>(
        &mut self,
        executor: E,
        item_id: i32,
        amount: i32,
    ) -> Result<Option<ItemAmount>, Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let current_item = match self.inventory.get(&item_id) {
            Some(item) => item.clone(),
            None => return Ok(None),
        };

        let current_amount = current_item.amount;
        if current_amount < amount {
            return Err("No hay suficientes items para remover".into());
        }

        let new_amount = current_amount - amount;

        if new_amount > 0 {
            sqlx::query!(
                "UPDATE normal_inventory
                SET amount = amount - $3
                WHERE item = $1 AND member = $2",
                item_id,
                self.id,
                amount
            )
            .execute(executor)
            .await?;
        } else {
            sqlx::query!(
                "DELETE FROM normal_inventory
                WHERE item = $1 AND member = $2",
                item_id,
                self.id
            )
            .execute(executor)
            .await?;
        }

        let removed_item = if new_amount > 0 {
            if let Some(item) = self.inventory.get_mut(&item_id) {
                item.amount = new_amount;
            }

            let mut item_copy = current_item.clone();
            item_copy.amount = amount;
            Some(item_copy)
        } else {
            let removed = self.inventory.remove(&item_id);

            if amount == current_amount {
                removed
            } else {
                let mut item_copy = current_item.clone();
                item_copy.amount = amount;
                Some(item_copy)
            }
        };

        Ok(removed_item)
    }

    pub fn get_inventory(&self) -> Vec<ItemAmount> {
        self.inventory.values().cloned().collect()
    }

    pub fn get_item_by_id(&self, item_id: i32) -> Option<ItemAmount> {
        self.inventory.get(&item_id).cloned()
    }
}
