using System;
using System.Runtime.InteropServices;

namespace CryptoApp.Interop
{
    public class RusterBot
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void DataCallback(IntPtr jsonPtr);

        [DllImport("rust_binance_text", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr start_bot(
            [MarshalAs(UnmanagedType.LPStr)] string coins,
            [MarshalAs(UnmanagedType.LPStr)] string streamTypes,
            ref int keepRunning,
            DataCallback callback);

        [DllImport("rust_binance_text", CallingConvention = CallingConvention.Cdecl)]
        private static extern void free_string(IntPtr s);

        private DataCallback? _callbackDelegate; 
        private int _keepRunning;

        public bool IsRunning => _keepRunning != 0;

        public event Action<string>? OnDataReceived;

        public string? LastError { get; private set; }

        public bool StartBot(string coins, string streamTypes)
        {
            if (IsRunning)
                return false;

            _keepRunning = 1;
            _callbackDelegate = new DataCallback(OnDataReceivedFromRust);

            IntPtr errPtr = start_bot(coins, streamTypes, ref _keepRunning, _callbackDelegate);

            if (errPtr != IntPtr.Zero)
            {
                LastError = Marshal.PtrToStringAnsi(errPtr);
                free_string(errPtr);
                return false;
            }

            LastError = null;
            return true;
        }

        public void StopBot()
        {
            _keepRunning = 0;
        }

        private void OnDataReceivedFromRust(IntPtr jsonPtr)
        {
            string? json = Marshal.PtrToStringAnsi(jsonPtr);
            if (!string.IsNullOrEmpty(json))
            {
                OnDataReceived?.Invoke(json);
            }
        }
    }
}
