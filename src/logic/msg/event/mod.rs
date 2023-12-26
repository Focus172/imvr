// #[derive(Debug)]
// pub enum FocusChange {
//     Gained,
//     Lost,
// }
//
// impl FocusChange {
//     #[inline]
//     pub fn did_gain(&self) -> bool {
//         matches!(self, Self::Gained)
//     }
// }
//
// impl From<bool> for FocusChange {
//     #[inline]
//     fn from(value: bool) -> Self {
//         match value {
//             true => FocusChange::Gained,
//             false => FocusChange::Lost,
//         }
//     }
// }
//
