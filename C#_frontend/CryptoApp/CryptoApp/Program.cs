using CryptoApp.Data;
using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.Components.Web;
using System.Diagnostics;

var logFilePath = $"console_log_{DateTime.Now:yyyyMMdd_HHmmssfff}.txt";
Console.SetOut(new StreamWriter(logFilePath, append: true) { AutoFlush = true });

var debugLogFile = $"debug_log_{DateTime.Now:yyyyMMdd_HHmmssfff}.txt";
var debugListener = new System.Diagnostics.TextWriterTraceListener(debugLogFile);

System.Diagnostics.Trace.Listeners.Add(debugListener);

System.Diagnostics.Trace.AutoFlush = true;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddRazorPages();
builder.Services.AddServerSideBlazor();
builder.Services.AddSingleton<WeatherForecastService>();

var app = builder.Build();

// Configure the HTTP request pipeline.
if (!app.Environment.IsDevelopment())
{
    app.UseExceptionHandler("/Error");
    // The default HSTS value is 30 days. You may want to change this for production scenarios, see https://aka.ms/aspnetcore-hsts.
    app.UseHsts();
}

app.UseHttpsRedirection();

app.UseStaticFiles();

app.UseRouting();

app.MapBlazorHub();
app.MapFallbackToPage("/_Host");

app.Run();
