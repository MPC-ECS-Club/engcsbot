use serenity::all::Member;



pub async fn is_user_admin(member: &Option<Box<Member>>) -> bool {
    if let Some(member) = member {
        if let Some(permissions) = member.permissions {
            return permissions.administrator();
        }
    }

    false
}