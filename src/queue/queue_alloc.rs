use crate::queue::{
    AsyncPrependQueue, AsyncQueue, AsyncReverseQueue, DoubleEndedQueue, PrependQueue, Queue,
    ReverseQueue,
};
use crate::ThreadSpawner;
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use simple_futures::complete_future::{CompleteFuture, CompleteFutureHandle};
use simple_futures::value_future::{ValueFuture, ValueFutureHandle};

/// A custom async queue for turing a queue asynchronous. Creates a task for
/// running the queue functions.
#[derive(Debug)]
pub struct AsyncCustomQueue<Q, MQ> {
    inner: Arc<AsyncQueueInner<Q, MQ>>,
}
impl<Q, MQ> AsyncCustomQueue<Q, MQ>
where
    Q: Queue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    /// Creates a new [`AsyncCustomQueue`] from a backing queue and message
    /// queue
    pub fn new<TS>(
        queue: Q,
        message_queue: MQ,
    ) -> Result<(Self, TS::ThreadHandle), TS::SpawnError>
    where
        TS: ThreadSpawner<()>,
    {
        let inner = Arc::new(AsyncQueueInner {
            queue,
            message_queue,
        });
        let weak_inner = Arc::downgrade(&inner);
        Ok((
            Self { inner },
            TS::try_spawn(move || Self::queue_task_function(weak_inner))?,
        ))
    }

    fn queue_task_function(inner: Weak<AsyncQueueInner<Q, MQ>>) {
        let mut read_queue = VecDeque::new();
        let mut write_queue = VecDeque::new();
        while let Some(inner) = inner.upgrade() {
            match inner.message_queue.pop() {
                AsyncQueueMessage::Write(write_operation) => {
                    if write_queue.is_empty() {
                        match write_operation {
                            WriteOperation::Push(append) => {
                                match handle_write_push(&inner.queue, append) {
                                    Ok(_) => read_loop(&inner.queue, &mut read_queue),
                                    Err(append) => {
                                        write_queue.push_back(WriteOperation::Push(append))
                                    }
                                }
                            }
                            WriteOperation::PushFront(_) => unreachable!(),
                        }
                    } else {
                        write_queue.push_back(write_operation);
                    }
                }
                AsyncQueueMessage::Read(read_operation) => {
                    if read_queue.is_empty() {
                        match read_operation {
                            ReadOperation::Pop(receive) => {
                                match handle_read_pop(&inner.queue, receive) {
                                    Ok(_) => write_loop(&inner.queue, &mut write_queue),
                                    Err(receive) => {
                                        read_queue.push_back(ReadOperation::Pop(receive))
                                    }
                                }
                            }
                            ReadOperation::PopBack(_) => unreachable!(),
                        }
                    }
                }
            }
        }
    }
}
impl<Q, MQ> Clone for AsyncCustomQueue<Q, MQ> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<Q, MQ> AsyncQueue for AsyncCustomQueue<Q, MQ>
where
    Q: Queue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type AsyncItem = Q::Item;
    type PushFuture = CompleteFuture;
    type PopFuture = ValueFuture<Self::AsyncItem>;

    fn push_async(&self, value: Self::AsyncItem) -> Self::PushFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Write(WriteOperation::Push(
                PushOperation {
                    future: future.get_handle(),
                    value,
                },
            )))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }

    fn pop_async(&self) -> Self::PopFuture {
        let future = ValueFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Read(ReadOperation::Pop(PopOperation(
                future.get_handle(),
            ))))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }
}

/// A custom async prepend queue for turing a prepend queue asynchronous.
/// Creates a task for running the queue functions.
#[derive(Debug)]
pub struct AsyncCustomPrependQueue<Q, MQ> {
    inner: Arc<AsyncQueueInner<Q, MQ>>,
}
impl<Q, MQ> AsyncCustomPrependQueue<Q, MQ>
where
    Q: PrependQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    /// Creates a new [`AsyncCustomPrependQueue`] from a backing queue and
    /// message queue
    pub fn new<TS>(
        queue: Q,
        message_queue: MQ,
    ) -> Result<(Self, TS::ThreadHandle), TS::SpawnError>
    where
        TS: ThreadSpawner<()>,
    {
        let inner = Arc::new(AsyncQueueInner {
            queue,
            message_queue,
        });
        let weak_inner = Arc::downgrade(&inner);
        Ok((
            Self { inner },
            TS::try_spawn(move || Self::queue_task_function(weak_inner))?,
        ))
    }

    fn queue_task_function(inner: Weak<AsyncQueueInner<Q, MQ>>) {
        let mut read_queue = VecDeque::new();
        let mut write_queue = VecDeque::new();
        while let Some(inner) = inner.upgrade() {
            match inner.message_queue.pop() {
                AsyncQueueMessage::Write(write_operation) => {
                    if write_queue.is_empty() {
                        match write_operation {
                            WriteOperation::Push(append) => {
                                match handle_write_push(&inner.queue, append) {
                                    Ok(_) => read_loop(&inner.queue, &mut read_queue),
                                    Err(append) => {
                                        write_queue.push_back(WriteOperation::Push(append))
                                    }
                                }
                            }
                            WriteOperation::PushFront(prepend) => {
                                match handle_write_push_front(&inner.queue, prepend) {
                                    Ok(_) => read_loop(&inner.queue, &mut read_queue),
                                    Err(prepend) => {
                                        write_queue.push_back(WriteOperation::PushFront(prepend))
                                    }
                                }
                            }
                        }
                    } else {
                        write_queue.push_back(write_operation);
                    }
                }
                AsyncQueueMessage::Read(read_operation) => {
                    if read_queue.is_empty() {
                        match read_operation {
                            ReadOperation::Pop(receive) => {
                                match handle_read_pop(&inner.queue, receive) {
                                    Ok(_) => write_prepend_loop(&inner.queue, &mut write_queue),
                                    Err(receive) => {
                                        read_queue.push_back(ReadOperation::Pop(receive))
                                    }
                                }
                            }
                            ReadOperation::PopBack(_) => unreachable!(),
                        }
                    }
                }
            }
        }
    }
}
impl<Q, MQ> Clone for AsyncCustomPrependQueue<Q, MQ> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<Q, MQ> AsyncQueue for AsyncCustomPrependQueue<Q, MQ>
where
    Q: PrependQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type AsyncItem = Q::Item;
    type PushFuture = CompleteFuture;
    type PopFuture = ValueFuture<Self::AsyncItem>;

    fn push_async(&self, value: Self::AsyncItem) -> Self::PushFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Write(WriteOperation::Push(
                PushOperation {
                    future: future.get_handle(),
                    value,
                },
            )))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }

    fn pop_async(&self) -> Self::PopFuture {
        let future = ValueFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Read(ReadOperation::Pop(PopOperation(
                future.get_handle(),
            ))))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }
}
impl<Q, MQ> AsyncPrependQueue for AsyncCustomPrependQueue<Q, MQ>
where
    Q: PrependQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type PushBackFuture = CompleteFuture;

    fn push_front_async(&self, value: Self::AsyncItem) -> Self::PushBackFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Write(WriteOperation::PushFront(
                PushFrontOperation {
                    future: future.get_handle(),
                    value,
                },
            )))
            .unwrap_or_else(|_| panic!("Could not add to message queue"));
        future
    }
}

/// A custom async reverse queue for turing a reverse queue asynchronous.
/// Creates a task for running the queue functions.
#[derive(Debug)]
pub struct AsyncCustomReverseQueue<Q, MQ> {
    inner: Arc<AsyncQueueInner<Q, MQ>>,
}
impl<Q, MQ> AsyncCustomReverseQueue<Q, MQ>
where
    Q: ReverseQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    /// Creates a new [`AsyncCustomReverseQueue`] from a backing queue and
    /// message queue
    pub fn new<TS>(
        queue: Q,
        message_queue: MQ,
    ) -> Result<(Self, TS::ThreadHandle), TS::SpawnError>
    where
        TS: ThreadSpawner<()>,
    {
        let inner = Arc::new(AsyncQueueInner {
            queue,
            message_queue,
        });
        let weak_inner = Arc::downgrade(&inner);
        Ok((
            Self { inner },
            TS::try_spawn(move || Self::queue_task_function(weak_inner))?,
        ))
    }

    fn queue_task_function(inner: Weak<AsyncQueueInner<Q, MQ>>) {
        let mut read_queue = VecDeque::new();
        let mut write_queue = VecDeque::new();
        while let Some(inner) = inner.upgrade() {
            match inner.message_queue.pop() {
                AsyncQueueMessage::Write(write_operation) => {
                    if write_queue.is_empty() {
                        match write_operation {
                            WriteOperation::Push(push) => {
                                match handle_write_push(&inner.queue, push) {
                                    Ok(_) => read_back_loop(&inner.queue, &mut read_queue),
                                    Err(append) => {
                                        write_queue.push_back(WriteOperation::Push(append))
                                    }
                                }
                            }
                            WriteOperation::PushFront(_) => unreachable!(),
                        }
                    } else {
                        write_queue.push_back(write_operation);
                    }
                }
                AsyncQueueMessage::Read(read_operation) => {
                    if read_queue.is_empty() {
                        match read_operation {
                            ReadOperation::Pop(pop) => match handle_read_pop(&inner.queue, pop) {
                                Ok(_) => write_loop(&inner.queue, &mut write_queue),
                                Err(pop) => read_queue.push_back(ReadOperation::Pop(pop)),
                            },
                            ReadOperation::PopBack(pop_back) => {
                                match handle_read_pop_back(&inner.queue, pop_back) {
                                    Ok(_) => write_loop(&inner.queue, &mut write_queue),
                                    Err(pop_back) => {
                                        read_queue.push_back(ReadOperation::PopBack(pop_back))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
impl<Q, MQ> Clone for AsyncCustomReverseQueue<Q, MQ> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<Q, MQ> AsyncQueue for AsyncCustomReverseQueue<Q, MQ>
where
    Q: ReverseQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type AsyncItem = Q::Item;
    type PushFuture = CompleteFuture;
    type PopFuture = ValueFuture<Self::AsyncItem>;

    fn push_async(&self, value: Self::AsyncItem) -> Self::PushFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Write(WriteOperation::Push(
                PushOperation {
                    future: future.get_handle(),
                    value,
                },
            )))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }

    fn pop_async(&self) -> Self::PopFuture {
        let future = ValueFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Read(ReadOperation::Pop(PopOperation(
                future.get_handle(),
            ))))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }
}
impl<Q, MQ> AsyncReverseQueue for AsyncCustomReverseQueue<Q, MQ>
where
    Q: ReverseQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type PopBackFuture = ValueFuture<Self::AsyncItem>;

    fn pop_back_async(&self) -> Self::PopBackFuture {
        let future = ValueFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Read(ReadOperation::PopBack(
                PopBackOperation(future.get_handle()),
            )))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }
}

/// A custom async double ended queue for turing a double ended queue
/// asynchronous. Creates a task for running the queue functions.
#[derive(Debug)]
pub struct AsyncCustomDoubleEndedQueue<Q, MQ> {
    inner: Arc<AsyncQueueInner<Q, MQ>>,
}
impl<Q, MQ> AsyncCustomDoubleEndedQueue<Q, MQ>
where
    Q: DoubleEndedQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    /// Creates a new [`AsyncCustomDoubleEndedQueue`] from a backing queue and
    /// message queue
    pub fn new<TS>(
        queue: Q,
        message_queue: MQ,
    ) -> Result<(Self, TS::ThreadHandle), TS::SpawnError>
    where
        TS: ThreadSpawner<()>,
    {
        let inner = Arc::new(AsyncQueueInner {
            queue,
            message_queue,
        });
        let weak_inner = Arc::downgrade(&inner);
        Ok((
            Self { inner },
            TS::try_spawn(move || Self::queue_task_function(weak_inner))?,
        ))
    }

    fn queue_task_function(inner: Weak<AsyncQueueInner<Q, MQ>>) {
        let mut read_queue = VecDeque::new();
        let mut write_queue = VecDeque::new();
        while let Some(inner) = inner.upgrade() {
            match inner.message_queue.pop() {
                AsyncQueueMessage::Write(write_operation) => {
                    if write_queue.is_empty() {
                        match write_operation {
                            WriteOperation::Push(append) => {
                                match handle_write_push(&inner.queue, append) {
                                    Ok(_) => read_back_loop(&inner.queue, &mut read_queue),
                                    Err(append) => {
                                        write_queue.push_back(WriteOperation::Push(append))
                                    }
                                }
                            }
                            WriteOperation::PushFront(prepend) => {
                                match handle_write_push_front(&inner.queue, prepend) {
                                    Ok(_) => read_back_loop(&inner.queue, &mut read_queue),
                                    Err(prepend) => {
                                        write_queue.push_back(WriteOperation::PushFront(prepend))
                                    }
                                }
                            }
                        }
                    } else {
                        write_queue.push_back(write_operation);
                    }
                }
                AsyncQueueMessage::Read(read_operation) => {
                    if read_queue.is_empty() {
                        match read_operation {
                            ReadOperation::Pop(receive) => {
                                match handle_read_pop(&inner.queue, receive) {
                                    Ok(_) => write_prepend_loop(&inner.queue, &mut write_queue),
                                    Err(receive) => {
                                        read_queue.push_back(ReadOperation::Pop(receive))
                                    }
                                }
                            }
                            ReadOperation::PopBack(pop_back) => {
                                match handle_read_pop_back(&inner.queue, pop_back) {
                                    Ok(_) => write_prepend_loop(&inner.queue, &mut write_queue),
                                    Err(pop_back) => {
                                        read_queue.push_back(ReadOperation::PopBack(pop_back))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
impl<Q, MQ> Clone for AsyncCustomDoubleEndedQueue<Q, MQ> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<Q, MQ> AsyncQueue for AsyncCustomDoubleEndedQueue<Q, MQ>
where
    Q: DoubleEndedQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type AsyncItem = Q::Item;
    type PushFuture = CompleteFuture;
    type PopFuture = ValueFuture<Self::AsyncItem>;

    fn push_async(&self, value: Self::AsyncItem) -> Self::PushFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Write(WriteOperation::Push(
                PushOperation {
                    future: future.get_handle(),
                    value,
                },
            )))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }

    fn pop_async(&self) -> Self::PopFuture {
        let future = ValueFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Read(ReadOperation::Pop(PopOperation(
                future.get_handle(),
            ))))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }
}
impl<Q, MQ> AsyncPrependQueue for AsyncCustomDoubleEndedQueue<Q, MQ>
where
    Q: DoubleEndedQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type PushBackFuture = CompleteFuture;

    fn push_front_async(&self, value: Self::AsyncItem) -> Self::PushBackFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Write(WriteOperation::PushFront(
                PushFrontOperation {
                    future: future.get_handle(),
                    value,
                },
            )))
            .unwrap_or_else(|_| panic!("Could not add to message queue"));
        future
    }
}
impl<Q, MQ> AsyncReverseQueue for AsyncCustomDoubleEndedQueue<Q, MQ>
where
    Q: DoubleEndedQueue + Send + Sync + 'static,
    MQ: Queue<Item = AsyncQueueMessage<Q::Item>> + Send + Sync + 'static,
{
    type PopBackFuture = ValueFuture<Self::AsyncItem>;

    fn pop_back_async(&self) -> Self::PopBackFuture {
        let future = ValueFuture::new();
        self.inner
            .message_queue
            .try_push(AsyncQueueMessage::Read(ReadOperation::PopBack(
                PopBackOperation(future.get_handle()),
            )))
            .unwrap_or_else(|_| panic!("Could not add the queue message queue!"));
        future
    }
}

#[derive(Debug)]
struct AsyncQueueInner<Q, MQ> {
    queue: Q,
    message_queue: MQ,
}
/// An internal message for the async custom queues
#[derive(Debug)]
pub enum AsyncQueueMessage<T> {
    /// An operation that adds to the queue
    Write(WriteOperation<T>),
    /// An operation that reads from the queue
    Read(ReadOperation<T>),
}
/// A push operation. Synonymous with [`Queue::push`].
#[derive(Debug)]
pub struct PushOperation<T> {
    future: CompleteFutureHandle,
    value: T,
}
/// A push front operation. Synonymous with [`PrependQueue::push_front`].
#[derive(Debug)]
pub struct PushFrontOperation<T> {
    future: CompleteFutureHandle,
    value: T,
}
/// An operation that adds to the queue
#[derive(Debug)]
pub enum WriteOperation<T> {
    /// A push operation. Synonymous with [`Queue::push`].
    Push(PushOperation<T>),
    /// A push front operation. Synonymous with [`PrependQueue::push_front`].
    PushFront(PushFrontOperation<T>),
}
/// A pop operation. Synonymous with [`Queue::pop`].
#[derive(Debug)]
pub struct PopOperation<T>(ValueFutureHandle<T>);
/// A pop back operation. Synonymous with [`ReverseQueue::pop_back`].
#[derive(Debug)]
pub struct PopBackOperation<T>(ValueFutureHandle<T>);
/// An operation that adds to the queue
#[derive(Debug)]
pub enum ReadOperation<T> {
    /// A pop operation. Synonymous with [`Queue::pop`].
    Pop(PopOperation<T>),
    /// A pop back operation. Synonymous with [`ReverseQueue::pop_back`].
    PopBack(PopBackOperation<T>),
}

fn handle_write_push<Q>(
    queue: &Q,
    mut append: PushOperation<Q::Item>,
) -> Result<(), PushOperation<Q::Item>>
where
    Q: Queue,
{
    match queue.try_push(append.value) {
        Ok(_) => match append.future.complete() {
            None | Some(false) => Ok(()),
            Some(true) => panic!("Future already completed!"),
        },
        Err(value) => {
            append.value = value;
            Err(append)
        }
    }
}
fn handle_write_push_front<Q>(
    queue: &Q,
    mut prepend: PushFrontOperation<Q::Item>,
) -> Result<(), PushFrontOperation<Q::Item>>
where
    Q: PrependQueue,
{
    match queue.try_push_front(prepend.value) {
        Ok(_) => match prepend.future.complete() {
            None | Some(false) => Ok(()),
            Some(true) => panic!("Future already completed!"),
        },
        Err(value) => {
            prepend.value = value;
            Err(prepend)
        }
    }
}
fn handle_read_pop<Q>(
    queue: &Q,
    receive: PopOperation<Q::Item>,
) -> Result<(), PopOperation<Q::Item>>
where
    Q: Queue,
{
    if let Some(value) = queue.try_pop() {
        if let Some(val) = receive.0.assign(value) {
            val.unwrap_or_else(|_| panic!("Could not set future!"))
        }
        Ok(())
    } else {
        Err(receive)
    }
}
fn handle_read_pop_back<Q>(
    queue: &Q,
    receive: PopBackOperation<Q::Item>,
) -> Result<(), PopBackOperation<Q::Item>>
where
    Q: ReverseQueue,
{
    if let Some(value) = queue.try_pop_back() {
        if let Some(val) = receive.0.assign(value) {
            val.unwrap_or_else(|_| panic!("Could not set future!"))
        }
        Ok(())
    } else {
        Err(receive)
    }
}

fn write_loop<Q>(queue: &Q, write_queue: &mut VecDeque<WriteOperation<Q::Item>>)
where
    Q: Queue,
{
    'WriteLoop: while let Some(write_operation) = write_queue.pop_front() {
        match write_operation {
            WriteOperation::Push(append) => {
                if let Err(append) = handle_write_push(queue, append) {
                    write_queue.push_front(WriteOperation::Push(append));
                    break 'WriteLoop;
                }
            }
            WriteOperation::PushFront(_) => unreachable!(),
        }
    }
}
fn write_prepend_loop<Q>(queue: &Q, write_queue: &mut VecDeque<WriteOperation<Q::Item>>)
where
    Q: PrependQueue,
{
    'WriteLoop: while let Some(write_operation) = write_queue.pop_front() {
        match write_operation {
            WriteOperation::Push(append) => {
                if let Err(append) = handle_write_push(queue, append) {
                    write_queue.push_front(WriteOperation::Push(append));
                    break 'WriteLoop;
                }
            }
            WriteOperation::PushFront(prepend) => {
                if let Err(prepend) = handle_write_push_front(queue, prepend) {
                    write_queue.push_front(WriteOperation::PushFront(prepend));
                    break 'WriteLoop;
                }
            }
        }
    }
}

fn read_loop<Q>(queue: &Q, read_queue: &mut VecDeque<ReadOperation<Q::Item>>)
where
    Q: Queue,
{
    'ReadLoop: while let Some(read_operation) = read_queue.pop_front() {
        match read_operation {
            ReadOperation::Pop(receive) => {
                if let Err(receive) = handle_read_pop(queue, receive) {
                    read_queue.push_front(ReadOperation::Pop(receive));
                    break 'ReadLoop;
                }
            }
            ReadOperation::PopBack(_) => unreachable!(),
        }
    }
}
fn read_back_loop<Q>(queue: &Q, read_queue: &mut VecDeque<ReadOperation<Q::Item>>)
where
    Q: ReverseQueue,
{
    'ReadLoop: while let Some(read_operation) = read_queue.pop_front() {
        match read_operation {
            ReadOperation::Pop(receive) => {
                if let Err(receive) = handle_read_pop(queue, receive) {
                    read_queue.push_front(ReadOperation::Pop(receive));
                    break 'ReadLoop;
                }
            }
            ReadOperation::PopBack(receive_back) => {
                if let Err(receive_back) = handle_read_pop_back(queue, receive_back) {
                    read_queue.push_front(ReadOperation::PopBack(receive_back));
                    break 'ReadLoop;
                }
            }
        }
    }
}
