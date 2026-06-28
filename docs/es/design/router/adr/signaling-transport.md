# Transporte de Señalización — Socket Unix Dual / TCP

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

El cliente de señalización se conecta a un servidor de retransmisión para intercambiar ofertas/respuestas SDP de WebRTC y candidatos ICE. Originalmente usaba solo sockets de dominio Unix (`tokio::net::UnixStream`), que no están disponibles en Windows. Para soporte multiplataforma, se necesita un respaldo.

## Decisión

Implementar un enum de doble transporte `SignalingStream` que prueba primero el socket de dominio Unix (en plataformas que lo admiten, detectado por el formato de ruta — comienza con `/` o termina con `.sock`), recurriendo a TCP. El adaptador `TransportWriter` abstrae sobre `AsyncWrite` para ambos transportes. En plataformas no Unix, solo TCP está disponible.

## Consecuencias

### Positivas

- Señalización multiplataforma; mismo protocolo JSON-RPC sobre ambos transportes

### Negativas

- La señalización TCP no está cifrada por defecto
- Los usuarios de Windows deben asegurar enlace solo a loopback
