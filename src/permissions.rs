
#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum UserPermissions {
    PRODUCT_READ = 0b00000001,
    PRODUCT_WRITE = 0b00000010,
    ORDER_READ = 0b00000100,
    ORDER_WRITE = 0b00001000,
    USER_READ = 0b00010000,
    USER_WRITE = 0b00100000,
    ADMIN = 0xFFFFFFFF,
}

impl std::ops::BitOr for UserPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self as u32 | rhs as u32) }
    }
}

impl From<u32> for UserPermissions {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl UserPermissions {
    pub fn has_permission(&self, permission: &UserPermissions) -> bool {
        let permission = *permission as u32;

        (*self as u32 & permission) == permission
    }
}