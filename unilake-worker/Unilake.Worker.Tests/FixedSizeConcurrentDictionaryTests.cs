using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Unilake.Worker.Tests
{
    [TestClass]
    public class FixedSizeConcurrentDictionaryTests
    {
        [TestMethod]
        public void Constructor_InvalidSizeLimit_ThrowsArgumentOutOfRangeException()
        {
            Action act = () => new FixedSizeConcurrentDictionary<string, int>(0);

            act.Should().Throw<ArgumentOutOfRangeException>()
                .WithMessage("Size limit must be greater than 0 (Parameter 'sizeLimit')");
        }

        [TestMethod]
        public void Add_AddsItemToDictionary()
        {
            var dictionary = new FixedSizeConcurrentDictionary<string, int>(3);

            dictionary.Add("key1", 1);
            dictionary.TryGetValue("key1", out var value);

            value.Should().Be(1);
        }

        [TestMethod]
        public void Add_ExceedingSizeLimit_RemovesOldestItem()
        {
            var dictionary = new FixedSizeConcurrentDictionary<string, int>(3);

            dictionary.Add("key1", 1);
            dictionary.Add("key2", 2);
            dictionary.Add("key3", 3);
            dictionary.Add("key4", 4);

            dictionary.TryGetValue("key1", out var removedValue).Should().BeFalse();
            removedValue.Should().Be(default);
        }

        [TestMethod]
        public void SetValue_UpdatesExistingValue()
        {
            var dictionary = new FixedSizeConcurrentDictionary<string, int>(3);

            dictionary.Add("key1", 1);
            dictionary.SetValue("key1", 100);
            dictionary.TryGetValue("key1", out var value);

            value.Should().Be(100);
        }

        [TestMethod]
        public void SetValue_NonExistingKey_DoesNotUpdate()
        {
            var dictionary = new FixedSizeConcurrentDictionary<string, int>(3);

            dictionary.SetValue("key1", 100);
            dictionary.TryGetValue("key1", out _).Should().BeFalse();
        }
    }
}
