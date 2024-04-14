use std::ops::BitOr;

#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, ts_rs::TS)]
#[ts(export)]
#[repr(u32)]
pub enum UserPermissionEnum {
    INVENTORY_READ = 0b0001,
    INVENTORY_WRITE = 0b0010,
    ORDER_READ = 0b0100,
    ORDER_WRITE = 0b1000,
    MANAGE_DB = 0b0100_0000,
    ADMIN = 0xFFFF_FFFF,
}

/// Update this array when adding new permissions.
/// Used to split permissions into individual permissions.
const PERMISSION_VARIANTS: &'static [UserPermissionEnum] = &[
    UserPermissionEnum::INVENTORY_READ,
    UserPermissionEnum::INVENTORY_WRITE,
    UserPermissionEnum::ORDER_READ,
    UserPermissionEnum::ORDER_WRITE,
    UserPermissionEnum::MANAGE_DB,
    UserPermissionEnum::ADMIN,
];

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct UserPermissions(pub u32);

impl UserPermissions {
    pub fn has_permissions(&self, permissions: UserPermissions) -> bool {
        self.0 & permissions.0 == permissions.0
    }

    pub fn split_into_vec(&self) -> UserPermissionVec {
        UserPermissionVec::split_from(*self)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, ts_rs::TS)]
#[serde(transparent)]
#[ts(export)]
pub struct UserPermissionVec(Vec<UserPermissionEnum>);

impl UserPermissionVec {
    /// Create a new UserPermissionVec from a list of permissions.
    /// This will remove duplicates.
    pub fn new(permissions: Vec<UserPermissionEnum>) -> Self {
        // Flatten the permissions to remove duplicates
        UserPermissionVec::split_from(UserPermissionVec(permissions).flatten())
    }

    pub fn split_from(permission: UserPermissions) -> Self {
        UserPermissionVec(
            UserPermissionEnum::variants()
                .iter()
                .filter(|p| permission.0 & **p as u32 == **p as u32)
                .copied()
                .collect(),
        )
    }

    pub fn has_permission(&self, permission: &UserPermissionEnum) -> bool {
        self.0.iter().any(|p| p.has_permission(permission))
    }

    pub fn flatten(&self) -> UserPermissions {
        self.0.iter().fold(UserPermissions(0), |acc, p| {
            UserPermissions(acc.0 | *p as u32)
        })
    }
}

impl From<UserPermissionVec> for UserPermissions {
    fn from(permissions: UserPermissionVec) -> Self {
        permissions.flatten()
    }
}

impl From<u32> for UserPermissions {
    fn from(permission: u32) -> Self {
        UserPermissions(permission)
    }
}

impl From<u32> for UserPermissionVec {
    fn from(permission: u32) -> Self {
        UserPermissionVec::split_from(UserPermissions(permission))
    }
}

impl Into<u32> for UserPermissionEnum {
    fn into(self) -> u32 {
        self as u32
    }
}

impl BitOr for UserPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl UserPermissionEnum {
    pub const fn has_permission(&self, permission: &UserPermissionEnum) -> bool {
        (*self as u32 & *permission as u32) == *permission as u32
    }

    pub const fn variants() -> &'static [UserPermissionEnum] {
        PERMISSION_VARIANTS
    }
}
