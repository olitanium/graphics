pub struct Config<'a> {
    pub gl_version: (u32, u32),
    pub screen_dims: (u32, u32),
    pub title: &'a str,
    pub full_screen: bool,
    pub mouse_fix_to_centre: bool,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            gl_version: (3, 3),
            screen_dims: (320, 480),
            title: "Graphics Engine by O. Harris",
            full_screen: false,
            mouse_fix_to_centre: false,
        }
    }
}
