# Comunicación por Puerto Serie mediante aoba

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight necesita soporte de puerto serie para gestión de dispositivos embebidos y sondeo de protocolo industrial (Modbus RTU). El crate hermano `aoba` ya proporciona enumeración de puertos serie multiplataforma, extracción de VID/PID/serial y funcionalidad de maestro Modbus RTU/TCP.

## Decisión

Delegar todas las operaciones de puerto serie a aoba. El módulo `serial` de Evernight define tipos (`SerialConfig`, `SerialPortInfo`) e implementa `TerminalBackend` sobre un transporte serie, llamando a aoba para la E/S real del puerto. La autodetección de protocolo (barrido de baudios/paridad Modbus RTU) también se delega a aoba.

## Consecuencias

### Positivas

- Reutiliza código probado; aoba maneja casos límite multiplataforma.

### Negativas

- Añade una dependencia de aoba; la funcionalidad serial requiere que aoba esté disponible.
