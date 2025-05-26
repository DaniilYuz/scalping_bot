using System.Runtime.InteropServices;

namespace CryptoApp.Interop
{
    public class Ruster
    {
        public long RusterCall(long a, long b)
        {
            return add(a, b);
        }
        [DllImport("rust_binance_text")]
        public static extern long add(long a, long b);
    }
}
