# Smell 9: Typed Error Boundary, Not String Buckets

This smell is about one very specific reduction:

- before: internal code keeps doing `map_err(|err| ... err.to_string())`, `map_err(|err| format!(...))`, or `Err(format!(...))`
- after: Alter rust code to enable use of ? by adding #[from] stuff to this error
