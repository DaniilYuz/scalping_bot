using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace CryptoApp.Interop
{
    public class RusterBot : IDisposable
    {
        private Process? _rustProcess;
        private StreamWriter? _consoleOutputWriter;
        private readonly string _consoleLogPath = $"rust_console_{DateTime.Now:yyyyMMdd_HHmmss}.log";

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

        [DllImport("rust_binance_text", CallingConvention = CallingConvention.Cdecl)]
        [return: MarshalAs(UnmanagedType.I1)] 
        public static extern bool is_runtime_initialized();

        private DataCallback? _callbackDelegate;
        private int _keepRunning;
        private GCHandle _callbackHandle;
        private bool _disposed = false;
        private readonly object _lock = new object();

        public bool IsRunning => _keepRunning != 0;
        public event Action<string>? OnDataReceived;
        public string? LastError { get; private set; }

        public bool StartBot(string coins, string streamTypes)
        {
            lock (_lock)
            {
                if (IsRunning || _disposed)
                    return false;

                try
                {
                    _keepRunning = 1;
                    _callbackDelegate = new DataCallback(OnDataReceivedFromRust);

                    _callbackHandle = GCHandle.Alloc(_callbackDelegate);

                    _consoleOutputWriter = new StreamWriter(_consoleLogPath, append: true) { AutoFlush = true };
                    Console.SetOut(_consoleOutputWriter);

                    IntPtr errPtr = start_bot(coins, streamTypes, ref _keepRunning, _callbackDelegate);

                    if (errPtr != IntPtr.Zero)
                    {
                        LastError = Marshal.PtrToStringAnsi(errPtr);
                        free_string(errPtr);

                        // Освобождаем handle только при ошибке
                        if (_callbackHandle.IsAllocated)
                            _callbackHandle.Free();

                        _keepRunning = 0;
                        return false;
                    }

                    LastError = null;
                    return true;
                }
                catch (Exception ex)
                {
                    LastError = $"C# Exception: {ex.Message}";

                    if (_callbackHandle.IsAllocated)
                        _callbackHandle.Free();

                    _keepRunning = 0;
                    return false;
                }
            }
        }

        public void StopBot()
        {
            lock (_lock)
            {
                if (!IsRunning)
                    return;

                try
                {
                    _keepRunning = 0;

                    // Даем время Rust коду завершиться
                    System.Threading.Thread.Sleep(1000);

                    if (_callbackHandle.IsAllocated)
                    {
                        _callbackHandle.Free();
                    }
                }
                catch (Exception ex)
                {
                    LastError = $"Stop error: {ex.Message}";
                }

                _consoleOutputWriter?.Dispose();
                _consoleOutputWriter = null;
            }
        }

        private void OnDataReceivedFromRust(IntPtr jsonPtr)
        {
            try
            {
                if (jsonPtr == IntPtr.Zero)
                    return;

                string? json = Marshal.PtrToStringAnsi(jsonPtr);
                if (!string.IsNullOrEmpty(json))
                {
                    // Вызываем событие в UI потоке если нужно
                    OnDataReceived?.Invoke(json);
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Callback error: {ex.Message}");
            }
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                if (disposing)
                {
                    StopBot();
                }
                _disposed = true;
            }
        }

        ~RusterBot()
        {
            Dispose(false);
        }
    }
}
