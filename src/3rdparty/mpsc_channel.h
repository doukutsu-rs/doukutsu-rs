/*
mpsc_channel.hpp
A simple C++ implementation of the 'channel' in rust language (std::mpsc).

# Usage
Use `mpsc::make_channel<T>` to create a channel. `make_channel<T>` will return a tuple of (Sender<T>, Receiver<T>).

For example:
```c++
// Create.
auto [ sender, receiver ] = mpsc::make_channel<int>();

// Send.
sender.send(3);

// Receive (both returns a std::optional<T>.)
receiver.receive(); // Blocking when there is nothing present in the channel.
receiver.try_receive(); // Not blocking. Return immediately.

// close() and closed()
sender.close();
bool result = sender.closed();
assert(result == receiver.closed());

// You can use range-based for loop to receive from the channel.
for (int v: receiver) {
    // do something with v
    // The loop will stop immedately after the sender called close().
    // Only sender can call close().
}
```

Note: `mpsc` stands for Multi-Producer Single-Consumer. So `Sender` can be either copied and moved, but `Receiver` can only be moved.

Feel free to explore the `tests.cpp`. The tests are also examples of the usage.

Read the source if you need more information. Sorry for the lack of comments. ～(￣▽￣～)~

Copyright (c) 2019 liuchibing.
*/

#pragma once

#include <list>
#include <queue>
#include <mutex>
#include <optional>
#include <tuple>
#include <memory>
#include <utility>
#include <condition_variable>
#include <exception>
#include <type_traits>
#include <iterator>

namespace mpsc
{

    template <typename T>
    class Sender;
    template <typename T>
    class Receiver;

    template <typename T>
    std::tuple<Sender<T>, Receiver<T>> make_channel();

    class channel_closed_exception : std::logic_error
    {
    public:
        channel_closed_exception() : std::logic_error("This channel has been closed.") {}
    };

    template <typename T>
    class Channel
    { // Do NOT use this class directly.
    public:
        void send(T &&value);
        void send(const T &value);

        std::optional<T> receive();
        std::optional<T> try_receive();

        void close();
        bool closed();

        Channel(const Channel<T> &) = delete;
        Channel(Channel<T> &&) = delete;

        Channel<T> &operator=(const Channel<T> &) = delete;
        Channel<T> &operator=(Channel<T> &&) = delete;

    private:
        Channel(){};

        std::queue<T, std::list<T>> queue;
        std::mutex mutex;
        std::condition_variable condvar;
        bool need_notify = false;
        bool _closed = false;

        friend std::tuple<Sender<T>, Receiver<T>> make_channel<T>();
    };

    template <typename T>
    class Sender
    {
    public:
        Sender<T> &send(T &&value)
        {
            validate();
            channel->send(std::move(value));
            return *this;
        }
        Sender<T> &send(const T &value)
        {
            validate();
            channel->send(value);
            return *this;
        }

        void close()
        {
            validate();
            channel->close();
        }

        bool closed()
        {
            validate();
            return channel->closed();
        }

        operator bool() const { return static_cast<bool>(channel); }

        Sender(const Sender<T> &) = default;
        Sender(Sender<T> &&) = default;

        Sender<T> &operator=(const Sender<T> &) = default;
        Sender<T> &operator=(Sender<T> &&) = default;

    private:
        Sender(std::shared_ptr<Channel<T>> channel) : channel(channel){};

        std::shared_ptr<Channel<T>> channel;

        void validate()
        {
            if (!channel)
            {
                throw std::invalid_argument("This sender has been moved out.");
            }
        }

        friend std::tuple<Sender<T>, Receiver<T>> make_channel<T>();
    };

    template <typename T>
    class Receiver
    {
    public:
        std::optional<T> receive()
        {
            validate();
            return channel->receive();
        }

        std::optional<T> try_receive()
        {
            validate();
            return channel->try_receive();
        }

        bool closed()
        {
            validate();
            return channel->closed();
        }

        operator bool() const
        {
            return static_cast<bool>(channel);
        }

        Receiver(Receiver<T> &&) = default;
        Receiver<T> &operator=(Receiver<T> &&) = default;

        Receiver(const Receiver<T> &) = delete;
        Receiver<T> &operator=(const Receiver<T> &) = delete;

    private:
        Receiver(std::shared_ptr<Channel<T>> channel) : channel(channel){};

        std::shared_ptr<Channel<T>> channel;

        void validate()
        {
            if (!channel)
            {
                throw std::invalid_argument("This receiver has been moved out.");
            }
        }

        friend std::tuple<Sender<T>, Receiver<T>> make_channel<T>();

    public:
        class iterator
            : public std::iterator<std::input_iterator_tag, T>
        {
        private:
            typedef std::iterator<std::input_iterator_tag, T> BaseIter;

        public:
            using typename BaseIter::difference_type;
            using typename BaseIter::iterator_category;
            using typename BaseIter::pointer;
            using typename BaseIter::reference;
            using typename BaseIter::value_type;

            iterator() : receiver(nullptr) {}
            iterator(Receiver<T> &receiver) : receiver(&receiver)
            {
                if (this->receiver->closed())
                    this->receiver = nullptr;
                else
                    next();
            }

            reference operator*() { return current.value(); }
            pointer operator->() { return &current.value(); }
            iterator &operator++()
            {
                next();
                return *this;
            }
            iterator operator++(int) = delete;
            bool operator==(iterator &other)
            {
                if (receiver == nullptr && other.receiver == nullptr)
                    return true;
                return false;
            }
            bool operator!=(iterator &other)
            {
                return !(*this == other);
            }

        private:
            Receiver<T> *receiver;
            std::optional<T> current = std::nullopt;
            void next();
        };

        iterator begin()
        {
            return iterator(*this);
        }

        iterator end()
        {
            return iterator();
        }
    };

    /* ======== Implementations ========= */

    template <typename T>
    std::tuple<Sender<T>, Receiver<T>> make_channel()
    {
        static_assert(std::is_copy_constructible_v<T> || std::is_move_constructible_v<T>, "T should be copy-constructible or move-constructible.");
        std::shared_ptr<Channel<T>> channel{new Channel<T>()};
        Sender<T> sender{channel};
        Receiver<T> receiver{channel};
        return std::tuple<Sender<T>, Receiver<T>>{
            std::move(sender),
            std::move(receiver)};
    }

    template <typename T>
    void Channel<T>::send(T &&value)
    {
        std::unique_lock lock(mutex);
        if (_closed)
        {
            throw channel_closed_exception();
        }
        queue.push(std::move(value));
        if (need_notify)
        {
            need_notify = false;
            lock.unlock();
            condvar.notify_one();
        }
    }

    template <typename T>
    void Channel<T>::send(const T &value)
    {
        std::unique_lock lock(mutex);
        if (_closed)
        {
            throw channel_closed_exception();
        }
        queue.push(value);
        if (need_notify)
        {
            need_notify = false;
            lock.unlock();
            condvar.notify_one();
        }
    }

    template <typename T>
    std::optional<T> Channel<T>::receive()
    {
        std::unique_lock lock(mutex);
        if (_closed)
        {
            return std::nullopt;
        }
        if (queue.empty())
        {
            need_notify = true;
            condvar.wait(lock, [this]
                         { return !queue.empty() || _closed; });
        }
        if (_closed)
        {
            return std::nullopt;
        }
        T result = std::move(queue.front());
        queue.pop();
        return result;
    }

    template <typename T>
    std::optional<T> Channel<T>::try_receive()
    {
        if (mutex.try_lock())
        {
            std::unique_lock lock(mutex, std::adopt_lock);
            if (_closed)
                return std::nullopt;
            if (queue.empty())
                return std::nullopt;
            T result = std::move(queue.front());
            queue.pop();
            return result;
        }
        return std::nullopt;
    }

    template <typename T>
    void Channel<T>::close()
    {
        std::unique_lock lock(mutex);
        _closed = true;
        if (need_notify)
        {
            need_notify = false;
            lock.unlock();
            condvar.notify_one();
        }
    }

    template <typename T>
    bool Channel<T>::closed()
    {
        std::unique_lock lock(mutex);
        return _closed;
    }

    template <typename T>
    void Receiver<T>::iterator::next()
    {
        if (!receiver)
            return;
        while (true)
        {
            if (receiver->closed())
            {
                receiver = nullptr;
                current.reset();
                return;
            }
            std::optional<T> tmp = receiver->receive();
            if (!tmp.has_value())
                continue;
            current.emplace(std::move(tmp.value()));
            return;
        }
    }

}
