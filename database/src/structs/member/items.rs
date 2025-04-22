use super::{Error, Member};
use crate::{
    models::{ItemType, Quality},
    structs::{InventoryItem, Item},
};
use std::collections::hash_map::Entry;

impl Member {
    pub async fn add_item_rpg<'a, E>(
        &mut self,
        executor: E,
        item: Item,
        amount: i32,
    ) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + std::marker::Copy,
    {
        if !self.in_rpg() {
            return Err("not in rpg".into());
        }

        let rpg = super::get_actual_rpg(executor).await?;

        if rpg.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "INSERT INTO player_inventory (rpg, item, player, amount)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (rpg, item, player) DO UPDATE
            SET amount = player_inventory.amount + $3",
            rpg.unwrap(),
            item.id,
            self.id,
            amount
        )
        .execute(executor)
        .await?;

        if let Some(ref mut inventory) = self.inventories.rpg {
            match inventory.items.entry(item.id.unwrap()) {
                Entry::Occupied(mut entry) => {
                    let current_item = entry.get_mut();
                    current_item.amount += amount;
                }
                Entry::Vacant(entry) => {
                    let mut new_item = InventoryItem { info: item, amount };
                    new_item.amount = amount;
                    entry.insert(new_item);
                }
            }
        }

        Ok(())
    }

    pub async fn remove_item_rpg<'a, E>(
        &mut self,
        executor: E,
        item_id: i32,
        amount: i32,
    ) -> Result<Option<InventoryItem>, Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + std::marker::Copy,
    {
        if !self.in_rpg() {
            return Err("not in rpg".into());
        }

        let rpg = super::get_actual_rpg(executor).await?;

        if rpg.is_none() {
            return Err("rpg not found".into());
        }

        let current_item = match self.inventories.rpg.clone().unwrap().items.get(&item_id) {
            Some(item) => item.clone(),
            None => return Ok(None),
        };

        if current_item.amount < amount {
            return Err("No hay suficientes items para remover".into());
        }

        let new_amount = current_item.amount - amount;

        if new_amount > 0 {
            sqlx::query!(
                "UPDATE player_inventory
                SET amount = amount - $3
                WHERE item = $1 AND player = $2 AND rpg = $4",
                item_id,
                self.id,
                amount,
                rpg
            )
            .execute(executor)
            .await?;
        } else {
            sqlx::query!(
                "DELETE FROM player_inventory
                WHERE item = $1 AND player = $2 AND rpg = $3",
                item_id,
                self.id,
                rpg
            )
            .execute(executor)
            .await?;
        }

        let removed_item = if new_amount > 0 {
            if let Some(ref mut inventory) = self.inventories.rpg {
                if let Some(item) = inventory.items.get_mut(&item_id) {
                    item.amount = new_amount;
                }
            }

            let mut item_copy = current_item.clone();
            item_copy.amount = amount;
            Some(item_copy)
        } else if let Some(ref mut inventory) = self.inventories.rpg {
            let removed = inventory.items.remove(&item_id);

            if amount == current_item.amount {
                removed
            } else {
                let mut item_copy = current_item.clone();
                item_copy.amount = amount;
                Some(item_copy)
            }
        } else {
            None
        };

        Ok(removed_item)
    }

    pub fn get_item(&self, item_name: String) -> Result<InventoryItem, Error> {
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

        match self.inventories.normal.items.entry(item.id.unwrap()) {
            Entry::Occupied(mut entry) => {
                let current_item = entry.get_mut();
                current_item.amount += amount;
            }
            Entry::Vacant(entry) => {
                let mut new_item = InventoryItem { info: item, amount };
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
    ) -> Result<Option<InventoryItem>, Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let current_item = match self.inventories.normal.items.get(&item_id) {
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
            if let Some(item) = self.inventories.normal.items.get_mut(&item_id) {
                item.amount = new_amount;
            }

            let mut item_copy = current_item.clone();
            item_copy.amount = amount;
            Some(item_copy)
        } else {
            let removed = self.inventories.normal.items.remove(&item_id);

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

    pub fn get_inventory(&self) -> Vec<InventoryItem> {
        let mut inventory: Vec<InventoryItem> =
            self.inventories.normal.items.values().cloned().collect();

        if let Some(ref club) = self.club {
            if let Some(ref item) = club.role.item {
                inventory.push(InventoryItem {
                    info: Item {
                        id: None,
                        name: item.item_tr_key.clone(),
                        number: None,
                        usable: false,
                        item_type: ItemType::Misc,
                        quality: Quality::Masterpiece,
                        armor_type: None,
                        tool_type: None,
                        two_handed: false,
                    },
                    amount: 1,
                })
            }
        }

        inventory
    }

    pub fn get_item_by_id_rpg(&self, item_id: i32) -> Option<InventoryItem> {
        self.inventories
            .rpg
            .clone()
            .map(|item| item.items.get(&item_id).cloned())?
    }

    pub fn get_item_by_id(&self, item_id: i32) -> Option<InventoryItem> {
        self.inventories.normal.items.get(&item_id).cloned()
    }
}
