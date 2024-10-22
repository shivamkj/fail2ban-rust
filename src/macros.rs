#[macro_export]
macro_rules! trn {
  ($test:expr => $true_expr:expr ; $false_expr:expr) => {
    if $test {
      $true_expr
    } else {
      $false_expr
    }
  };
}

#[macro_export]
macro_rules! expr {
  ($test:expr, $err:expr) => {
    match $test {
      Ok(res) => res,
      Err(_) => $err,
    }
  };
}

#[macro_export]
macro_rules! expo {
  ($test:expr, $none:expr) => {
    match $test {
      Some(res) => res,
      None => $none,
    }
  };
}
