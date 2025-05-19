use crate::{
    error::ClubError,
    models::{AuthorityId, ClubItemType, ClubRolePerm, ClubType},
    PgPool,
};
use sqlx::FromRow;

#[derive(Clone, FromRow)]
pub struct ClubRole {
    pub tr_key: String,
    pub authority_id: Option<AuthorityId>,
    pub authority: i32,
    pub member_limit: i32,
    pub perms: Vec<ClubRolePerm>,
    pub item_tr_key: Option<String>,
}

#[derive(Clone, FromRow)]
pub struct ClubStlRules {
    pub required_role: Option<String>,
    pub required_balance: Option<i64>,
    pub karamete: Option<bool>,
    pub required_agent_zero: Option<bool>,
}

#[derive(Clone)]
pub struct ClubMember {
    pub id: i64,
    pub role_tr_key: String,
    pub nth: i32,
}

#[derive(Clone)]
pub struct Club {
    pub id: i64,
    pub name: String,
    pub members: Vec<ClubMember>,
    pub roles: Vec<ClubRole>,
    pub prestige: i32,
    pub description: Option<String>,
    pub renameable: bool,
    pub deleteable: bool,
    pub bank: i64,
    pub points: i32,
    pub club_type: ClubType,
    pub stl_rules: ClubStlRules,
}

impl Club {
    pub async fn build(pool: &PgPool, club_id: i64) -> Result<Self, ClubError> {
        let (stl_rules, club_info, roles, members_record) = tokio::try_join!(
            sqlx::query_as!(
                ClubStlRules,
                "SELECT required_role, required_balance, karamete, required_agent_zero
                FROM club_stl_rules
                WHERE club = $1;",
                club_id
            )
            .fetch_one(pool),
            sqlx::query!(
                "SELECT renameable, deleteable, name, prestige, description, bank, points,
                club_type AS \"club_type: ClubType\"
                FROM club WHERE id = $1;",
                club_id
            )
            .fetch_one(pool),
            sqlx::query_as!(
                ClubRole,
                "SELECT cr.tr_key, cr.authority,
                cr.perms AS \"perms: Vec<ClubRolePerm>\",
                cr.authority_id AS \"authority_id: AuthorityId\",
                cr.item_tr_key,
                rl.member_limit
                FROM club_role cr
                JOIN club_limits rl ON rl.role_name = cr.tr_key AND rl.club = cr.club
                WHERE cr.club = $1;",
                club_id
            )
            .fetch_all(pool),
            sqlx::query!(
                r#"SELECT
                    m.id,
                    cr.tr_key,
                    crl.nth
                FROM club_member cm
                JOIN member m ON m.id = cm.member
                JOIN club_role_log crl ON crl.member = m.id
                JOIN club_role cr ON cr.club = cm.club
                WHERE cm.club = $1
                ORDER BY crl.assigned_at DESC;"#,
                club_id
            )
            .fetch_all(pool),
        )?;

        let members = members_record
            .into_iter()
            .map(|record| ClubMember {
                id: record.id,
                role_tr_key: record.tr_key,
                nth: record.nth,
            })
            .collect();

        Ok(Self {
            id: club_id,
            name: club_info.name,
            members,
            roles,
            prestige: club_info.prestige,
            description: club_info.description,
            renameable: club_info.renameable,
            deleteable: club_info.deleteable,
            bank: club_info.bank,
            points: club_info.points,
            club_type: club_info.club_type,
            stl_rules,
        })
    }

    pub async fn set_stl_rules(
        &mut self,
        pool: &PgPool,
        required_role_tr_key: Option<String>,
        required_balance: Option<i64>,
        required_agent_zero: Option<bool>,
        karamete: Option<bool>,
    ) -> Result<(), ClubError> {
        sqlx::query!(
            "UPDATE club_stl_rules SET required_role = $1, required_balance = $2, required_agent_zero = $3, karamete = $4 WHERE club = $5;",
            required_role_tr_key,
            required_balance,
            required_agent_zero,
            karamete,
            self.id
        )
        .execute(pool)
        .await?;

        self.stl_rules.required_role = required_role_tr_key;
        self.stl_rules.required_balance = required_balance;
        self.stl_rules.karamete = karamete;
        self.stl_rules.required_agent_zero = required_agent_zero;

        Ok(())
    }

    pub async fn create(pool: &PgPool, leader_id: i64, name: String) -> Result<Self, ClubError> {
        sqlx::query!(
            "SELECT create_club($1, $2, $3, $4, $5);",
            leader_id,
            name,
            "leader",
            "agent",
            "member"
        )
        .fetch_optional(pool)
        .await?;

        let club_id = sqlx::query!("SELECT id FROM club WHERE name = $1;", name)
            .fetch_one(pool)
            .await?
            .id;

        Self::build(pool, club_id).await
    }

    pub async fn delete(&mut self, pool: &PgPool) -> Result<(), ClubError> {
        if !self.deleteable {
            return Err(ClubError::NotDeletable);
        }

        sqlx::query!("DELETE FROM club WHERE id = $1;", self.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub fn get_member(&self, member_id: i64) -> Option<&ClubMember> {
        self.members.iter().find(|m| m.id == member_id)
    }

    pub fn get_role(&self, role_tr_key: String) -> Option<&ClubRole> {
        self.roles.iter().find(|r| r.tr_key == role_tr_key)
    }

    pub fn is_leader(&self, member_id: i64) -> bool {
        if let Some(member) = self.get_member(member_id) {
            if let Some(role) = self
                .roles
                .iter()
                .find(|role| role.tr_key == member.role_tr_key)
            {
                return role.authority_id == Some(AuthorityId::Leader);
            }
        }
        false
    }

    pub fn get_leader(&self) -> Option<&ClubMember> {
        self.members.iter().find(|m| {
            if let Some(role) = self.roles.iter().find(|role| role.tr_key == m.role_tr_key) {
                role.authority_id == Some(AuthorityId::Leader)
            } else {
                false
            }
        })
    }

    pub fn total_members(&self) -> usize {
        self.members.len()
    }

    pub async fn rename(&mut self, pool: &PgPool, name: String) -> Result<(), ClubError> {
        if !self.renameable {
            return Err(ClubError::NotRenameable);
        }

        sqlx::query!("UPDATE club SET name = $1 WHERE id = $2;", name, self.id)
            .execute(pool)
            .await?;

        self.name = name;

        Ok(())
    }

    pub async fn set_description(
        &mut self,
        pool: &PgPool,
        description: Option<String>,
    ) -> Result<(), ClubError> {
        sqlx::query!(
            "UPDATE club SET description = $1 WHERE id = $2;",
            description,
            self.id
        )
        .execute(pool)
        .await?;

        self.description = description;

        Ok(())
    }

    pub async fn set_type(&mut self, pool: &PgPool, club_type: ClubType) -> Result<(), ClubError> {
        sqlx::query!(
            "UPDATE club SET club_type = $1 WHERE id = $2;",
            club_type as _,
            self.id
        )
        .execute(pool)
        .await?;

        self.club_type = club_type;

        Ok(())
    }

    pub async fn increase_bank(&mut self, pool: &PgPool, amount: i64) -> Result<(), ClubError> {
        sqlx::query!(
            "UPDATE club SET bank = bank + $1 WHERE id = $2;",
            amount,
            self.id
        )
        .execute(pool)
        .await?;

        self.bank += amount;

        Ok(())
    }

    pub async fn decrease_bank(&mut self, pool: &PgPool, amount: i64) -> Result<(), ClubError> {
        if self.bank < amount {
            return Err(ClubError::InsufficientFunds);
        }

        sqlx::query!(
            "UPDATE club SET bank = bank - $1 WHERE id = $2;",
            amount,
            self.id
        )
        .execute(pool)
        .await?;

        self.bank -= amount;

        Ok(())
    }

    pub async fn increase_points(&mut self, pool: &PgPool, amount: i32) -> Result<(), ClubError> {
        sqlx::query!(
            "UPDATE club SET points = points + $1 WHERE id = $2;",
            amount,
            self.id
        )
        .execute(pool)
        .await?;

        self.points += amount;

        Ok(())
    }

    pub async fn create_role(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
        authority: i32,
        limit: i32,
    ) -> Result<(), ClubError> {
        if self.roles.iter().any(|role| role.tr_key == role_tr_key) {
            return Err(ClubError::RoleAlreadyExists);
        }

        let mut tx = pool.begin().await?;

        sqlx::query!(
            "INSERT INTO club_role (tr_key, authority, club) VALUES ($1, $2, $3);",
            role_tr_key,
            authority,
            self.id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO club_limits (role_name, member_limit, club) VALUES ($1, $2, $3);",
            role_tr_key,
            limit,
            self.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        self.roles.push(ClubRole {
            tr_key: role_tr_key,
            authority_id: None,
            authority,
            member_limit: limit,
            perms: vec![],
            item_tr_key: None,
        });

        Ok(())
    }

    pub async fn delete_role(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
    ) -> Result<(), ClubError> {
        match self.roles.iter().find(|role| role.tr_key == role_tr_key) {
            Some(role) => {
                if role.authority_id.is_some() {
                    return Err(ClubError::CannotDeleteRole);
                }
            }
            None => {
                return Err(ClubError::RoleNotFound);
            }
        }

        if let Some(pos) = self
            .roles
            .iter()
            .position(|role| role.tr_key == role_tr_key)
        {
            sqlx::query!(
                "DELETE FROM club_role WHERE tr_key = $1 AND club = $2;",
                role_tr_key,
                self.id
            )
            .execute(pool)
            .await?;

            self.roles.remove(pos);
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub async fn rename_role(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
        new_tr_key: String,
    ) -> Result<(), ClubError> {
        if self.roles.iter().any(|role| role.tr_key == new_tr_key) {
            return Err(ClubError::RoleAlreadyExists);
        }

        sqlx::query!(
            "UPDATE club_role SET tr_key = $1 WHERE tr_key = $2 AND club = $3;",
            new_tr_key,
            role_tr_key,
            self.id
        )
        .execute(pool)
        .await?;

        self.members.iter_mut().for_each(|member| {
            if member.role_tr_key == role_tr_key {
                member.role_tr_key = new_tr_key.clone();
            }
        });

        if let Some(role) = self
            .roles
            .iter_mut()
            .find(|role| role.tr_key == role_tr_key)
        {
            role.tr_key = new_tr_key;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub async fn role_set_perms(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
        perms: Vec<ClubRolePerm>,
    ) -> Result<(), ClubError> {
        if let Some(role) = self
            .roles
            .iter_mut()
            .find(|role| role.tr_key == role_tr_key)
        {
            sqlx::query!(
                "UPDATE club_role SET perms = $1 WHERE tr_key = $2 AND club = $3;",
                perms as _,
                role_tr_key,
                self.id
            )
            .execute(pool)
            .await?;

            role.perms = perms;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub async fn role_set_limit(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
        limit: i32,
    ) -> Result<(), ClubError> {
        if let Some(role) = self
            .roles
            .iter_mut()
            .find(|role| role.tr_key == role_tr_key)
        {
            sqlx::query!(
                "UPDATE club_limits SET member_limit = $1 WHERE role_name = $2 AND club = $3;",
                limit,
                role_tr_key,
                self.id
            )
            .execute(pool)
            .await?;

            role.member_limit = limit;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub async fn role_set_authority(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
        authority: i32,
    ) -> Result<(), ClubError> {
        if authority > 100 {
            return Err(ClubError::InvalidAuthority);
        }

        if self.roles.iter().any(|role| role.authority == authority) {
            return Err(ClubError::DuplicateAuthority);
        }

        if let Some(role) = self
            .roles
            .iter_mut()
            .find(|role| role.tr_key == role_tr_key)
        {
            sqlx::query!(
                "UPDATE club_role SET authority = $1 WHERE tr_key = $2 AND club = $3;",
                authority,
                role_tr_key,
                self.id
            )
            .execute(pool)
            .await?;

            role.authority = authority;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub fn count_members_with_role(&self, role_tr_key: String) -> Result<i32, ClubError> {
        if let Some(role) = self.roles.iter().find(|role| role.tr_key == role_tr_key) {
            Ok(self
                .members
                .iter()
                .filter(|m| m.role_tr_key == role.tr_key)
                .count() as i32)
        } else {
            Err(ClubError::RoleNotFound)
        }
    }

    pub async fn change_role(
        &mut self,
        pool: &PgPool,
        member_id: i64,
        role_tr_key: String,
        assigned_by: Option<i64>,
    ) -> Result<(), ClubError> {
        if self.members.iter().any(|m| m.id == member_id) {
            return Err(ClubError::MemberAlreadyHasRole);
        }

        if let Some(role) = self.roles.iter().find(|role| role.tr_key == role_tr_key) {
            if self.count_members_with_role(role_tr_key.clone())? >= role.member_limit {
                return Err(ClubError::MemberLimitReached);
            }

            sqlx::query!(
                "UPDATE club_member SET role_name = $1 WHERE member = $2 AND club = $3;",
                role_tr_key,
                member_id,
                self.id
            )
            .execute(pool)
            .await?;

            self.log_role_assign(pool, member_id, role_tr_key, assigned_by)
                .await?;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub async fn kick_member(&mut self, pool: &PgPool, member_id: i64) -> Result<(), ClubError> {
        if let Some(pos) = self.members.iter().position(|m| m.id == member_id) {
            sqlx::query!(
                "DELETE FROM club_member WHERE member = $1 AND club = $2;",
                member_id,
                self.id
            )
            .execute(pool)
            .await?;

            self.members.remove(pos);
        } else {
            return Err(ClubError::MemberNotFound);
        }

        Ok(())
    }

    pub async fn join_member(
        &mut self,
        pool: &PgPool,
        member_id: i64,
        role_tr_key: String,
        assigned_by: Option<i64>,
    ) -> Result<(), ClubError> {
        if self.members.iter().any(|m| m.id == member_id) {
            return Err(ClubError::MemberAlreadyExists);
        }

        if let Some(role) = self.roles.iter().find(|role| role.tr_key == role_tr_key) {
            if self.count_members_with_role(role_tr_key.clone())? >= role.member_limit {
                return Err(ClubError::MemberLimitReached);
            }

            sqlx::query!(
                "INSERT INTO club_member (member, role_name, club) VALUES ($1, $2, $3);",
                member_id,
                &role_tr_key,
                self.id
            )
            .execute(pool)
            .await?;

            self.members.push(ClubMember {
                id: member_id,
                role_tr_key: role_tr_key.clone(),
                nth: 0,
            });

            self.log_role_assign(pool, member_id, role_tr_key, assigned_by)
                .await?;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub fn can_manage_roles(&self, member_id: i64) -> bool {
        if let Some(member) = self.get_member(member_id) {
            if let Some(role) = self
                .roles
                .iter()
                .find(|role| role.tr_key == member.role_tr_key)
            {
                return role.perms.iter().any(|perm| {
                    matches!(perm, ClubRolePerm::ManageRoles) || matches!(perm, ClubRolePerm::All)
                });
            }
        }
        false
    }

    pub async fn edit_role_item(
        &mut self,
        pool: &PgPool,
        role_tr_key: String,
        item_tr_key: Option<String>,
    ) -> Result<(), ClubError> {
        if let Some(role) = self
            .roles
            .iter_mut()
            .find(|role| role.tr_key == role_tr_key)
        {
            sqlx::query!(
                "UPDATE club_role SET item_tr_key = $1 WHERE tr_key = $2 AND club = $3;",
                item_tr_key,
                role_tr_key,
                self.id
            )
            .execute(pool)
            .await?;

            role.item_tr_key = item_tr_key;
        } else {
            return Err(ClubError::RoleNotFound);
        }

        Ok(())
    }

    pub fn can_manage_members(&self, member_id: i64) -> bool {
        if let Some(member) = self.get_member(member_id) {
            if let Some(role) = self
                .roles
                .iter()
                .find(|role| role.tr_key == member.role_tr_key)
            {
                return role.perms.iter().any(|perm| {
                    matches!(perm, ClubRolePerm::ManageMembers) || matches!(perm, ClubRolePerm::All)
                });
            }
        }
        false
    }

    pub fn can_manage_club(&self, member_id: i64) -> bool {
        if let Some(member) = self.get_member(member_id) {
            if let Some(role) = self
                .roles
                .iter()
                .find(|role| role.tr_key == member.role_tr_key)
            {
                return role.perms.iter().any(|perm| {
                    matches!(perm, ClubRolePerm::ManageClub) || matches!(perm, ClubRolePerm::All)
                });
            }
        }
        false
    }

    pub fn can_manage_bank(&self, member_id: i64) -> bool {
        if let Some(member) = self.get_member(member_id) {
            if let Some(role) = self
                .roles
                .iter()
                .find(|role| role.tr_key == member.role_tr_key)
            {
                return role.perms.iter().any(|perm| {
                    matches!(perm, ClubRolePerm::ManageBank) || matches!(perm, ClubRolePerm::All)
                });
            }
        }
        false
    }

    pub async fn transfer(&mut self, pool: &PgPool, new_leader_id: i64) -> Result<(), ClubError> {
        let leader_role = self
            .roles
            .iter()
            .find(|role| role.authority_id == Some(AuthorityId::Leader))
            .ok_or(ClubError::RoleNotFound)?
            .clone();

        let agent_role = self
            .roles
            .iter()
            .find(|role| role.authority_id == Some(AuthorityId::Agent))
            .ok_or(ClubError::RoleNotFound)?
            .clone();

        let old_leader_id = self
            .members
            .iter()
            .find(|member| member.role_tr_key == leader_role.tr_key)
            .ok_or(ClubError::MemberNotFound)?
            .clone()
            .id;

        if self
            .change_role(pool, old_leader_id, agent_role.tr_key, None)
            .await
            .is_err()
        {
            self.kick_member(pool, old_leader_id)
                .await
                .map_err(|_| ClubError::MemberNotFound)?;
        }

        self.change_role(pool, new_leader_id, leader_role.tr_key, None)
            .await
            .map_err(|_| ClubError::MemberNotFound)?;

        Ok(())
    }

    pub async fn log_role_assign(
        &mut self,
        pool: &PgPool,
        member_id: i64,
        role_tr_key: String,
        assigned_by: Option<i64>,
    ) -> Result<(), ClubError> {
        let previus_role = match self.get_member(member_id) {
            Some(member) => &member.role_tr_key,
            None => return Err(ClubError::MemberNotFound),
        };

        if self.get_role(role_tr_key.clone()).is_none() {
            return Err(ClubError::RoleNotFound);
        }

        sqlx::query!(
            "SELECT log_club_role($1, $2, $3, $4, $5);",
            self.id,
            member_id,
            role_tr_key,
            assigned_by,
            previus_role
        )
        .execute(pool)
        .await?;

        if let Some(member) = self.members.iter_mut().find(|m| m.id == member_id) {
            let record = sqlx::query!("SELECT nth FROM club_role_log WHERE club = $1 AND member = $2 ORDER BY assigned_by DESC;",self.id, member_id).fetch_one(pool).await?;
            member.nth = record.nth;
        }

        Ok(())
    }
}
