pub struct QueueDefinition<'qe> {
    pub(crate) name: &'qe str,
}

pub struct QueueBinding<'qeb> {
    pub(crate) queue_name: &'qeb str,
}
