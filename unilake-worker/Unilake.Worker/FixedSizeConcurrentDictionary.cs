using System.Collections.Concurrent;

namespace Unilake.Worker;

public class FixedSizeConcurrentDictionary<TKey, TValue>
{
    private readonly int _sizeLimit;
    private readonly ConcurrentDictionary<TKey, TValue> _dictionary;
    private readonly ConcurrentQueue<TKey> _queue;

    public FixedSizeConcurrentDictionary(int sizeLimit)
    {
        if (sizeLimit <= 0)
            throw new ArgumentOutOfRangeException(nameof(sizeLimit), "Size limit must be greater than 0");

        _sizeLimit = sizeLimit;
        _dictionary = new ConcurrentDictionary<TKey, TValue>();
        _queue = new ConcurrentQueue<TKey>();
    }

    public void Add(TKey key, TValue value)
    {
        _queue.Enqueue(key);
        _dictionary[key] = value;

        if (_queue.Count > _sizeLimit && _queue.TryDequeue(out var oldestKey))
            _dictionary.TryRemove(oldestKey, out _);
    }

    public bool TryGetValue(TKey key, out TValue value) => 
        _dictionary.TryGetValue(key, out value);

    public void SetValue(TKey key, TValue value)
    {
        if (_dictionary.ContainsKey(key))
            _dictionary[key] = value;
    }
}