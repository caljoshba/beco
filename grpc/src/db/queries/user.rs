pub const SELECT: &str = "SELECT id, details, sequence_number FROM personal.user WHERE id=$1";
pub const INSERT: &str = "INSERT INTO personal.user (id, details, sequence_number) VALUES ($1, $2, $3);";
pub const UPDATE: &str = "UPDATE personal.user SET details=$2, sequence_number=$3 WHERE id=$1;";