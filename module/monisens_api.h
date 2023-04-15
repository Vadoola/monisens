#include "monisens_def.h"

// -------------------------------------------------------------------------------------------
// ------------------------------ Функции для инициализации ----------------------------------
// -------------------------------------------------------------------------------------------

// Метод инициализации обработчика модуля. Обработчик (handler) - это любая структура,
// используемая внутри модуля, описываемая разработчиком этого модуля и
// содержащая всю необходимую информацию для корректной работы.
// Память для обработчика выделяется и управляется внутри модуля.
// Для правильного освобождения памяти применяется функция destroy.
void init(void **handler);

// Функция получения параметров подключения. Она вызывает `callback` из аргумента
// и пердоставляет ему доступ к параметрам. `callback` должен скопировать значения
// из указателя на параметры подключения.
void obtain_device_info(void *handler, void *obj, device_info_callback callback);

// Функция подключения к устройству.
// Возвращает коды ошибок:
//   - 0 - успех,
//   - 1 - подключение неудачно,
//   - 2 - неверные параметры.
// Внутри этой функции модуль благодаря коммуникации с устройством может
// определить, какие параметры возвращать в функции `obtain_sensor_confs`.
// Также здесь следует выполнять проверку правильности конфигурации устройства.
uint8_t connect_device(void *handler, DeviceConnectConf *connect_conf);

// -------------------------------------------------------------------------------------------
// ------------------------- Функции для конфигурации устройства -----------------------------
// -------------------------------------------------------------------------------------------

// Получение параметров для конфигурации устройства
void obtain_device_conf_info(void *handler, void *obj, device_conf_info_callback callback);

// Конфигурация устройства на основе параметров из `obtain_device_conf_info`
// Возвращает коды ошибок:
//   - 0 - успех,
//   - 1 - подключение неудачно,
//   - 2 - неверные параметры.
uint8_t configure_device(void *handler, DeviceConf *conf);

// -------------------------------------------------------------------------------------------
// -------------------- Функции для получения информации об устройстве -----------------------
// -------------------------------------------------------------------------------------------

// Конфигурация устройства на основе параметров из `obtain_device_conf_info`
// Возвращает коды ошибок:
//   - 0 - успех,
//   - 1 - подключение неудачно.
// Система отдельно будет возвращать свои ошибки если имена сенсоров и их данных не пройдут
// валидацию.
uint8_t obtain_sensor_type_infos(void *handler, void *obj, sensor_type_infos_callback callback);

// -------------------------------------------------------------------------------------------
// ----------------------- Функции для коммуникации с устройством -------------------------
// -------------------------------------------------------------------------------------------

// Начать работу модуля.
//
// `msg_handler` может безопсано отправляться и копироваться между потоками.
uint8_t start(void *handler, void *msg_handler, handle_msg_func handle_func);

// Остановить работу модуля.
//
// После выполнения выполнения этого метода модуль должен гарантировать, что `msg_handler`
// и `handle_func`, переданные при вызове `start()`, были удалены из памяти модуля.
uint8_t stop(void *handler);

// -------------------------------------------------------------------------------------------
// ------------------------- Функции для процесса работы модуля ------------------------------
// -------------------------------------------------------------------------------------------

void destroy(void *handler);

// Метод, возвращающий используемую версию заголовка
// для совместимости со старыми версиями в будущем.
uint8_t mod_version();

// Метод, возвращающий все функции модуля
Functions functions();
