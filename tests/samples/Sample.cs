using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace Demo
{
    /// <summary>XML 문서 주석</summary>
    public class Sample : IDisposable
    {
        private readonly List<string> _items = new List<string>();
        public string Name { get; set; } = "Don Editor";
        public const int MaxCount = 10;

        public async Task<int> RunAsync(int count)
        {
            // 한 줄 주석
            for (int i = 0; i < count; i++)
            {
                if (i % 2 == 0 && Name != null)
                {
                    Console.WriteLine($"값: {i}");
                }
            }
            await Task.Delay(100);
            return count;
        }

        public void Dispose() { }

        public static void Main(string[] args)
        {
            var sample = new Sample();
            double ratio = 3.14;
            char c = 'x';
            bool ok = true;
            Console.WriteLine(ok ? "yes" : "no");
        }
    }
}
