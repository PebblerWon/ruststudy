# -*- coding:utf-8 -*-
__author__ = 'weijie,zzx'

"""
*   EmQuantAPI for python
*   version  2.7.7.0
*   Copyright(c)2016-2026,  Shanghai Choice Information Technology Co,. Ltd. All Rights Reserved. * 
"""
import os
import sys
import platform

if sys.platform.startswith("linux"):
    os_bit = platform.machine().strip().lower()
    if "aarch64" in os_bit or "arm64" in os_bit:
        base_dir = os.path.dirname(os.path.abspath(__file__))
        lib_path = os.path.join(base_dir, "libs", "linuxArm", "x64", "libEMQuantAPIx64.so")

        if "LD_PRELOAD" not in os.environ:
            os.environ["LD_PRELOAD"] = lib_path
            os.execv(sys.executable, [sys.executable] + sys.argv)
# ====================================================================================

from ctypes import *
import platform as _platform
import sys as _sys, os as _os
from datetime import datetime as _datetime, date as _date

OS_Window = 1
OS_Linux = 2
OS_Mac = 3

OS_Bit32 = 1
OS_Bit64 = 2

PY_Python2 = 1
PY_Python3 = 2

PY_Bit32 = 1
PY_Bit64 = 2

eVT_null = 0
eVT_char = 1
eVT_byte = 2
eVT_bool = 3
eVT_short = 4
eVT_ushort = 5
eVT_int = 6
eVT_uInt = 7
eVT_int64 = 8
eVT_uInt64 = 9
eVT_float = 10
eVT_double = 11
eVT_byteArray = 12
eVT_asciiString = 13
eVT_unicodeString = 14

ePT_NONE = 0        #不使用代理
ePT_HTTP = 1        #HTTP代理
ePT_HTTPS = 2       #HTTPS代理
ePT_SOCK4 = 3       #SOCK4代理
ePT_SOCK5 = 4       #SOCK5代理

eOT_default = 0      # 默认(默认则根据传入数量的正负标志买入eOT_buy卖出eOT_Sell,其余类型对数量作正负转换)
eOT_buy = 1          # 买入
eOT_sell = 2         # 卖出
eOT_purchase = 3     # 申购
eOT_redemption = 4   # 赎回

eCfnMode_StartToEnd = 1 # starttime和endtime中间的所有资讯
eCfnMode_EndCount = 2   # 提取endtime的近count条数据

class c_safe_union(Union):
    _fields_ = [
        ("charValue", c_char),
        ("boolValue", c_bool),
        ("shortValue", c_short),
        ("uShortValue", c_ushort),
        ("intValue", c_int),
        ("uIntValue", c_uint),
        ("int64Value", c_longlong),
        ("uInt64Value", c_ulonglong),
        ("floatValue", c_float),
        ("doubleValue", c_double)
    ]
class stEQChar(Structure):
    _fields_ = [
        ("pChar", c_char_p),
        ("nSize", c_uint)
    ]
class stEQCharArray(Structure):
    _fields_ = [
        ("pChArray", POINTER(stEQChar)),
        ("nSize", c_uint)
    ]
class stEQVarient(Structure):
    _fields_ = [
        ("vtype", c_int),
        ("unionValues", c_safe_union),
        ("eqchar", stEQChar)
    ]
class stEQVarientArray(Structure):
    _fields_ = [
        ("pEQVarient", POINTER(stEQVarient)),
        ("nSize", c_uint)
    ]
class stEQData(Structure):
    _fields_ = [
        ("codeArray", stEQCharArray),
        ("indicatorArray", stEQCharArray),
        ("dateArray", stEQCharArray),
        ("valueArray", stEQVarientArray)
    ]
class stEQLoginInfo(Structure):
    _fields_ = [
        ("userName", c_char * 255),
        ("password", c_char * 255)
    ]
class stEQMessage(Structure):
    _fields_ = [
        ("version", c_int),
        ("msgType", c_int),
        ("err", c_int),
        ("requestID", c_int),
        ("serialID", c_int),
        ("pEQData", POINTER(stEQData))
    ]
class stEQCtrData(Structure):
    _fields_ = [
        ("row", c_int),
        ("column", c_int),
        ("indicatorArray", stEQCharArray),
        ("valueArray", stEQVarientArray)
    ]
class stOrderInfo(Structure):
    _pack_=8
    _fields_ = [
        ("code", c_char*20),
        ("volume", c_double),
        ("price", c_float),
        ("date", c_int),
        ("time", c_int),
        ("optype", c_int),
        ("cost", c_float),
        ("rate", c_float),
        ("reserve", c_int)
    ]

class Adapter:
    def __init__(self):
        self.__os_name = OS_Window
        self.__os_bit  = OS_Bit32
        self.__py_name = PY_Python2
        self.__py_bit = PY_Bit32
        self.__os_arch = "x86_64"

        self.InitSysInfo()

    def InitSysInfo(self):
        osname = _platform.system()
        osbit = _platform.machine()
        pyname = _platform.python_version()
        pybit = _platform.architecture()[0]

        clean_osbit = osbit.strip().lower()
        if (osname == "Windows"):
            self.__os_name = OS_Window
        elif osname == "Linux":
            self.__os_name = OS_Linux
            self.__os_arch = "x86_64"
            if "aarch64" in clean_osbit or "arm64" in clean_osbit:
                self.__os_arch = "arm64"
        elif osname == "Darwin":
            self.__os_name = OS_Mac

        self.__os_bit = OS_Bit32
        if (self.__os_name == OS_Window and "amd64" in clean_osbit) or \
                (self.__os_name == OS_Linux and ("x86_64" in clean_osbit or "aarch64" in clean_osbit or "arm64" in clean_osbit)) or \
                (self.__os_name == OS_Mac and "x86_64" in clean_osbit):
            self.__os_bit = OS_Bit64

        if pyname[0] == "2":
            self.__py_name = PY_Python2
        elif pyname[0] == "3":
            self.__py_name = PY_Python3

        if pybit == "32bit":
            self.__py_bit = PY_Bit32
        elif pybit == "64bit":
            self.__py_bit = PY_Bit64

    def get_os_name(self):
        return self.__os_name
    def get_os_bit(self):
        return self.__os_bit
    def get_py_name(self):
        return self.__py_name
    def get_py_bit(self):
        return self.__py_bit
    def get_os_arch(self):
        return self.__os_arch

class UtilAccess:
    adapter = Adapter()

    @staticmethod
    def GetLibraryPath():
        os_name = UtilAccess.adapter.get_os_name()
        if os_name == OS_Window:
            return UtilAccess.__getLibraryPath_window()
        elif os_name == OS_Linux:
            return UtilAccess.__getLibraryPath_linux()
        elif os_name == OS_Mac:
            return UtilAccess.__getLibraryPath_mac()

    @staticmethod
    def GetEncodeType():
        if UtilAccess.adapter.get_os_name() == OS_Window:
            return "gbk"
        else:
            return "utf-8"
    @staticmethod
    def GetLanguageVersion():
        os_name = UtilAccess.adapter.get_os_name()
        if os_name == OS_Window:
            return "LANGUAGEVERSION=403"
        elif os_name == OS_Linux:
            return "LANGUAGEVERSION=503"
        elif os_name == OS_Mac:
            return "LANGUAGEVERSION=603"

    @staticmethod
    def __getLibraryPath_window():
        apiPackagePath = "."
        for x in _sys.path:
            xx = x.find("site-packages")
            if(xx >= 0 and x[xx:] == "site-packages"):
                apiPackagePath = x
                break

        apiPackagePath = _os.path.join(apiPackagePath, "EmQuantAPI.pth")
        if not _os.path.exists(apiPackagePath):
            return ""
        pthFile = open(apiPackagePath, "r")
        baseDir = pthFile.readline().strip()
        pthFile.close()

        apiDllPath = ""
        if baseDir != "":
            libsDir = _os.path.join(baseDir, "libs", "windows")
            if UtilAccess.adapter.get_py_bit() == PY_Bit32:
                apiDllPath = _os.path.join(libsDir, "EmQuantAPI.dll")
            else:
                apiDllPath = _os.path.join(libsDir, "EmQuantAPI_x64.dll")
        return apiDllPath

    @staticmethod
    def __getLibraryPath_linux():
        baseDir = ""

        site_pkg_names = ["site-packages", "dist-packages"]

        for site_pkg_name in site_pkg_names:
            if baseDir != "":
                break
            for spath in _sys.path:
                pos = spath.find(site_pkg_name)
                if(pos >= 0 and spath[pos:]==site_pkg_name):
                    apiPackagePath = _os.path.join(spath, "EmQuantAPI.pth")
                    if not _os.path.exists(apiPackagePath):
                        continue
                    pthFile = open(apiPackagePath, "r")
                    baseDir = pthFile.readline().strip()
                    pthFile.close()
                    break

        apiDllPath = ""
        if baseDir != "":
            libsDir = _os.path.join(baseDir, "libs", "linux")
            os_arch = UtilAccess.adapter.get_os_arch()
            if UtilAccess.adapter.get_py_bit() == PY_Bit32:
                apiDllPath = _os.path.join(libsDir, "x86", "libEMQuantAPI.so")
            else:
                if os_arch == "arm64":
                    apiDllPath = _os.path.join(baseDir, "libs", "linuxArm", "x64", "libEMQuantAPIx64.so")
                else:
                    apiDllPath = _os.path.join(libsDir, "x64", "libEMQuantAPIx64.so")
        return apiDllPath

    @staticmethod
    def __getLibraryPath_mac():
        baseDir = ""
        site_pkg_name = "site-packages"
        for x in _sys.path:
            xi = x.find(site_pkg_name)
            if(xi >= 0 and x[xi:] == site_pkg_name):
                apiPackagePath = x
                apiPackagePath = _os.path.join(apiPackagePath, "EmQuantAPI.pth")
                if not _os.path.exists(apiPackagePath):
                    continue
                pthFile = open(apiPackagePath, "r")
                baseDir = pthFile.readline().strip()
                pthFile.close()
                if len(baseDir) > 0:
                    break
        apiDllPath = ""
        if baseDir != "":
            apiDllPath = _os.path.join(baseDir, "libs", "mac", "libEMQuantAPIx64.dylib")
        return apiDllPath


def DemoCallback(quantdata):
    """
    DemoCallback 是csq订阅时提供的回调函数模板。该函数只有一个为c.EmQuantData类型的参数quantdata
    :param quantdata:cls.EmQuantData
    :return:
    """
    print("QuoteCallback,", str(quantdata))

def chqDemoCallback(quantdata):
    """
    chqDemoCallback 是chq订阅时提供的回调函数模板。该函数只有一个为c.EmQuantData类型的参数quantdata
    :param quantdata:cls.EmQuantData
    :return:
    """
    print("chqQuoteCallback,", str(quantdata))

def cstCallBack(quantdata):
    '''
    cstCallBack 是日内跳价服务提供的回调函数模板
    '''
    if isinstance(quantdata.Data, list):
        for code, codeData in zip(quantdata.Codes, quantdata.Data):
            print(code)
            length = len(quantdata.Dates)
            for k in range(0, length):
                for j in range(0, len(quantdata.Indicators)):
                    print(codeData[j * length + k], " ", end = "")
                print()
        return
    for i in range(0, len(quantdata.Codes)):
        length = len(quantdata.Dates)
        for it in quantdata.Data.keys():
            print(it)
            for k in range(0, length):
                for j in range(0, len(quantdata.Indicators)):
                    print(quantdata.Data[it][j * length + k], " ", end = "")
                print()

def cnqdemoCallBack(quantdata):
    """
        cnqdemoCallBack 是cnq订阅时提供的回调函数模板。该函数只有一个为c.EmQuantData类型的参数quantdata
        :param quantdata:c.EmQuantData
        :return:
        """
    # print("InfoCallback,", str(quantdata))
    print("cnqCallback,")
    for code in quantdata.Data:
        total = len(quantdata.Data[code])
        for k in range(0, len(quantdata.Data[code])):
            print(quantdata.Data[code][k])
class c:
    class EmQuantData:
        def __init__(self, NullValue = None):
            self.ErrorCode = 0
            self.ErrorMsg = "success"
            self.Codes = list()
            self.Indicators = list()
            self.Dates = list()
            self.RequestID = 0
            self.SerialID = 0
            self.Data = dict()
            # broker=(重复代码)场景下保留的原始记录：[(code, record), ...]
            # record = [指标0的(日期0..n), 指标1的(日期0..n), ...]，与 self.Codes 一一对应
            self.Records = []
            # Records 里每条记录的排布是否按日期成行（True=日期在最外层）
            self.RecordGroupByDate = False
            self.__NullValue = NullValue

        def __str__(self):
            return "ErrorCode=%s, ErrorMsg=%s, Data=%s" % (self.ErrorCode, self.ErrorMsg, str(self.Data))

        def __repr__(self):
            return "ErrorCode=%s, ErrorMsg=%s, Data=%s" % (self.ErrorCode, self.ErrorMsg, str(self.Data))

        def resolve2RankData(self, indicatorData, **arga):
            for i in range(0, indicatorData.codeArray.nSize):
                self.Codes.append(indicatorData.codeArray.pChArray[i].pChar.decode(c.EncodeType))
            for k in range(0, indicatorData.indicatorArray.nSize):
                self.Indicators.append(indicatorData.indicatorArray.pChArray[k].pChar.decode(c.EncodeType))
            for j in range(0, indicatorData.dateArray.nSize):
                self.Dates.append(indicatorData.dateArray.pChArray[j].pChar.decode(c.EncodeType))

            self.Data = []

            for i in range(0, len(self.Codes)):
                for j in range(0, len(self.Indicators)):
                    for k in range(0, len(self.Dates)):
                        self.Data.append(self.getIndicatorDataByIndex(i, j, k, indicatorData))

        def resolve25RankData(self, indicatorData, **arga):
            for i in range(0, indicatorData.codeArray.nSize):
                self.Codes.append(indicatorData.codeArray.pChArray[i].pChar.decode(c.EncodeType))
            for k in range(0, indicatorData.indicatorArray.nSize):
                self.Indicators.append(indicatorData.indicatorArray.pChArray[k].pChar.decode(c.EncodeType))
            for j in range(0, indicatorData.dateArray.nSize):
                self.Dates.append(indicatorData.dateArray.pChArray[j].pChar.decode(c.EncodeType))

            if arga.get("keepDuplicateCodes", False):
                # 默认（groupByDate=False）按指标成块：record = [指标0的(日期0..n), 指标1的(日期0..n), ...]
                # 即 record[indicatorIndex * 日期数 + dateIndex]，与无 broker、10004/10005/10006 异步一致
                # groupByDate=True 时改成按日期成行，仅作为可选项保留（当前没有调用方使用）
                groupByDate = arga.get("groupByDate", False)
                records = []
                for i in range(0, len(self.Codes)):
                    record = []
                    if groupByDate:
                        for k in range(0, len(self.Dates)):
                            for j in range(0, len(self.Indicators)):
                                record.append(self.getIndicatorDataByIndex(i, j, k, indicatorData))
                    else:
                        for j in range(0, len(self.Indicators)):
                            for k in range(0, len(self.Dates)):
                                record.append(self.getIndicatorDataByIndex(i, j, k, indicatorData))
                    records.append(record)
                # 用 DuplicateCodeData 包一层：仍是 list，但兼容 .items()/.keys()/.get() 等旧写法
                self.Data = c.DuplicateCodeData(self.Codes, records)
                # 原始记录留一份，供 applyLegacyDuplicateShape 还原旧布局、recordsOf 取单条记录
                self.Records = list(zip(self.Codes, records))
                self.RecordGroupByDate = groupByDate
                return

            for i in range(0, len(self.Codes)):
                stockCode = self.Codes[i]
                self.Data[stockCode] = []
                for j in range(0, len(self.Indicators)):
                    tempData = None
                    for k in range(0, len(self.Dates)):
                        tempData = self.getIndicatorDataByIndex(i, j, k, indicatorData)
                        self.Data[stockCode].append(tempData)

        def applyLegacyDuplicateShape(self, asDict=False):
            """
            broker=(重复代码)场景下，Data 内部是按"每条记录、指标成块"存放的：
                Data[recordIndex] = [指标0的(日期0..n), 指标1的(日期0..n), ...]
            这种扁平布局适合内部转 pandas，但 ispandas=0 直接返回时，老代码仍按
                Data[i][j]            (单代码：指标 i 的第 j 个日期值)
                Data[code][i][j]      (多代码：某代码指标 i 的第 j 个日期值)
            取值会 IndexError / TypeError。这里还原成普通路径的形态，且不丢重复记录：
            同一代码的多条记录按服务端返回顺序首尾拼接，
            第 r 条记录落在 j ∈ [r * 日期数, (r + 1) * 日期数) 区间内。

            :param asDict: True=多代码(dict，Data[code] -> [[指标0的日期值...], ...])
                           False=单代码(list，Data -> [[指标0的日期值...], ...])
            """
            nInd = len(self.Indicators)
            nDate = len(self.Dates)
            records = self.Records if len(self.Records) > 0 else list(zip(self.Codes, self.Data))

            def valueAt(record, j, k):
                # 取记录里"指标 j、日期下标 k"的值，兼容两种内部排布
                if self.RecordGroupByDate:
                    return record[k * nInd + j]
                return record[j * nDate + k]

            def appendInto(target, record):
                for k in range(0, nDate):
                    for j in range(0, nInd):
                        target[j].append(valueAt(record, j, k))

            if asDict:
                legacy = dict()
                for code, record in records:
                    if code not in legacy:
                        legacy[code] = [[] for j in range(0, nInd)]
                    appendInto(legacy[code], record)
                self.Data = legacy
            else:
                legacy = [[] for j in range(0, nInd)]
                for code, record in records:
                    appendInto(legacy, record)
                self.Data = legacy

        def recordsOf(self, code):
            """
            取某代码在 broker=(重复代码)场景下的全部记录。

            :return: [[[指标0的日期值...], [指标1的日期值...]], ...]  按服务端返回顺序
                     每条记录 = [[指标0的日期值...], [指标1的日期值...]]
            """
            nInd = len(self.Indicators)
            nDate = len(self.Dates)
            result = []
            for item in self.Records:
                if item[0] != code:
                    continue
                record = item[1]
                block = []
                for j in range(0, nInd):
                    if self.RecordGroupByDate:
                        block.append([record[k * nInd + j] for k in range(0, nDate)])
                    else:
                        block.append([record[j * nDate + k] for k in range(0, nDate)])
                result.append(block)
            return result

        def resolve25RankDataEx(self, indicatorData, **arga):
            for i in range(0, indicatorData.codeArray.nSize):
                self.Codes.append(indicatorData.codeArray.pChArray[i].pChar.decode(c.EncodeType))
            for k in range(0, indicatorData.indicatorArray.nSize):
                self.Indicators.append(indicatorData.indicatorArray.pChArray[k].pChar.decode(c.EncodeType))
            for j in range(0, indicatorData.dateArray.nSize):
                self.Dates.append(indicatorData.dateArray.pChArray[j].pChar.decode(c.EncodeType))
            for i in range(0, len(self.Codes)):
                stockCode = self.Codes[i]
                if not (stockCode in self.Data.keys()):
                    self.Data[stockCode] = []
                thislist = []
                for j in range(0, len(self.Indicators)):
                    tempData = None
                    for k in range(0, len(self.Dates)):
                        tempData = self.getIndicatorDataByIndex(i, j, k, indicatorData)
                        thislist.append(tempData)
                self.Data[stockCode].append(thislist)

        def resolve26RankData(self, indicatorData, **arga):
            for i in range(0, indicatorData.codeArray.nSize):
                self.Codes.append(indicatorData.codeArray.pChArray[i].pChar.decode(c.EncodeType))
            for k in range(0, indicatorData.indicatorArray.nSize):
                self.Indicators.append(indicatorData.indicatorArray.pChArray[k].pChar.decode(c.EncodeType))
            for j in range(0, indicatorData.dateArray.nSize):
                self.Dates.append(indicatorData.dateArray.pChArray[j].pChar.decode(c.EncodeType))
            self.Data = []
            for i in range(0, len(self.Codes)):
                for j in range(0, len(self.Indicators)):
                    tempData = []
                    for k in range(0, len(self.Dates)):
                        tempData.append(self.getIndicatorDataByIndex(i, j, k, indicatorData))
                    self.Data.append(tempData)


        def resolve3RankData(self, indicatorData, **arga):
            for i in range(0, indicatorData.codeArray.nSize):
                self.Codes.append(indicatorData.codeArray.pChArray[i].pChar.decode(c.EncodeType))
            for k in range(0, indicatorData.indicatorArray.nSize):
                self.Indicators.append(indicatorData.indicatorArray.pChArray[k].pChar.decode(c.EncodeType))
            for j in range(0, indicatorData.dateArray.nSize):
                self.Dates.append(indicatorData.dateArray.pChArray[j].pChar.decode(c.EncodeType))
            for i in range(0, len(self.Codes)):
                stockCode = self.Codes[i]
                self.Data[stockCode] = []
                for j in range(0, len(self.Indicators)):
                    tempData = []
                    for k in range(0, len(self.Dates)):
                        tempData.append(self.getIndicatorDataByIndex(i, j, k, indicatorData))
                    self.Data[stockCode].append(tempData)

        def resolveCtrData(self, indicatorData, **arga):
            for i in range(0, indicatorData.column):
                self.Indicators.append(indicatorData.indicatorArray.pChArray[i].pChar.decode(c.EncodeType))
            for r in range(0, indicatorData.row):
                list1 = []
                for n in range(0, indicatorData.column):
                    list1.append(self.resolve(indicatorData.valueArray.pEQVarient[indicatorData.column * r + n]))
                self.Data[str(r)] = list1

        def resolve(self, variant):
            if variant.vtype == eVT_null:
                return self.__NullValue
            elif variant.vtype == eVT_char:
                return variant.unionValues.charValue
            elif variant.vtype == eVT_bool:
                return variant.unionValues.boolValue
            elif variant.vtype == eVT_short:
                return variant.unionValues.shortValue
            elif variant.vtype == eVT_ushort:
                return variant.unionValues.uShortValue
            elif variant.vtype == eVT_int:
                return variant.unionValues.intValue
            elif variant.vtype == eVT_uInt:
                return variant.unionValues.uIntValue
            elif variant.vtype == eVT_int64:
                return variant.unionValues.int64Value
            elif variant.vtype == eVT_uInt64:
                return variant.unionValues.uInt64Value
            elif variant.vtype == eVT_float:
                return round(variant.unionValues.floatValue, 6)
            elif variant.vtype == eVT_double:
                return round(variant.unionValues.doubleValue, 6)
            elif variant.vtype == eVT_asciiString:
                if variant.eqchar.pChar != None:
                    return variant.eqchar.pChar.decode(c.EncodeType)
                else:
                    return ""
            elif variant.vtype == eVT_unicodeString:
                if variant.eqchar.pChar != None:
                    return variant.eqchar.pChar.decode(c.EncodeType)
                else:
                    return ""
            return self.__NullValue

        def getIndicatorDataByIndex(self, codeIndex, indicatorIndex, dateIndex, indicatorData):
            if indicatorData.valueArray.nSize == 0:
                return self.__NullValue
            codeSize = indicatorData.codeArray.nSize
            indicatorSize = indicatorData.indicatorArray.nSize
            dateSize = indicatorData.dateArray.nSize
            valueSize = indicatorData.valueArray.nSize
            if valueSize != codeSize * dateSize * indicatorSize:
                return self.__NullValue
            if codeIndex <= codeSize * indicatorSize * dateIndex + indicatorSize * codeIndex + indicatorIndex:
                tempIndex = codeSize * indicatorSize * dateIndex + indicatorSize * codeIndex + indicatorIndex
                return self.resolve(indicatorData.valueArray.pEQVarient[tempIndex])

    class DuplicateCodeData(list):
        """
        保留重复代码模式下的 Data 容器（broker= / keepDuplicateCodes=1 / 10004、10005 回调）。

        本质是 list：按服务端返回顺序保存每条记录，重复代码不会被合并。
        额外提供 dict 风格的 items()/keys()/values()/get()，让原来按 {代码: 记录} 写的代码
        不至于直接 AttributeError；重复代码会在 items() 里以相同 key 出现多次。

        - data.Data[i]              -> 第 i 条记录(list)
        - data.Data[code]           -> 该代码的第一条记录（等价 get，兼容旧的 Data[it] 写法）
        - data.Data.items()         -> [(code, record), ...]  重复代码会重复出现
        - data.Data.get(code)       -> 该代码的第一条记录，没有则 None
        - data.Data.getAll(code)    -> 该代码的全部记录(list of record)
        - data.Data.getCodes()      -> 去重后的代码列表(保持首次出现顺序)
        """
        def __init__(self, codes=None, records=None):
            list.__init__(self, records if records != None else [])
            self.Codes = list(codes) if codes != None else []

        def __getitem__(self, key):
            # 整数/切片仍按 list 语义；字符串按代码取第一条记录，
            # 这样 for it in Data.keys(): Data[it][...] 这种老写法不会 TypeError
            if isinstance(key, str):
                n = min(len(self.Codes), len(self))
                for i in range(0, n):
                    if self.Codes[i] == key:
                        return list.__getitem__(self, i)
                raise KeyError(key)
            return list.__getitem__(self, key)

        def keys(self):
            return list(self.Codes)

        def values(self):
            return [self[i] for i in range(0, len(self))]

        def items(self):
            n = min(len(self.Codes), len(self))
            return [(self.Codes[i], self[i]) for i in range(0, n)]

        def get(self, code, default=None):
            n = min(len(self.Codes), len(self))
            for i in range(0, n):
                if self.Codes[i] == code:
                    return self[i]
            return default

        def getAll(self, code):
            n = min(len(self.Codes), len(self))
            return [self[i] for i in range(0, n) if self.Codes[i] == code]

        def getCodes(self):
            result = []
            n = min(len(self.Codes), len(self))
            for i in range(0, n):
                if self.Codes[i] not in result:
                    result.append(self.Codes[i])
            return result

    EncodeType = ""
    Type_logOutFunc = CFUNCTYPE(c_int, c_char_p)
    Type_AsynDataFunc = CFUNCTYPE(c_int, POINTER(stEQMessage), c_void_p)

    __InitSucceed = False
    __apiDllPath = ""
    __quantLib = None
    __QuantFuncDict = {}

    __logOutFunc = None
    __AsynDataFunc = None
    __HandleAsynDataFuncDict = {0:{}, 10000:{}, 10001:{}, 10002:{}, 10003:{}, 10004:{}, 10005:{}, 10006:{}} # 0:main 10000:csq 10001:cst 10002:cnq 10003:chq 10004:csq(UnusualMov=1) 10005:csq broker 10006:cst broker

    __setCsqSerialID = set()
    __setChqSerialID = set()

    @classmethod
    def __Init(cls):
        if cls.__InitSucceed:
            return

        cls.EncodeType = UtilAccess.GetEncodeType()
        cls.__InitSucceed = True
        cls.__apiDllPath = UtilAccess.GetLibraryPath()
        cls.__quantLib = CDLL(cls.__apiDllPath)
        cls.__AsynDataFunc = cls.Type_AsynDataFunc(cls.__HandleAsynData)

        quantLib = cls.__quantLib

        quant_start = quantLib.start
        quant_start.restype = c_int
        quant_start.argtypes = [c_void_p, c_char_p, cls.Type_logOutFunc]

        quant_stop = quantLib.stop
        quant_stop.restype = c_int
        quant_stop.argtypes = []

        quant_setcallback = quantLib.setcallback
        quant_setcallback.restype = c_int
        quant_setcallback.argtypes = [cls.Type_AsynDataFunc]

        quant_geterrstring = quantLib.geterrstring
        quant_geterrstring.restype = c_char_p
        quant_geterrstring.argtypes = [c_int, c_int]

        quant_csd = quantLib.csd
        quant_csd.restype = c_int
        quant_csd.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p, c_void_p]

        quant_css = quantLib.css
        quant_css.restype = c_int
        quant_css.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_cses = quantLib.cses
        quant_cses.restype = c_int
        quant_cses.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_tradedates = quantLib.tradedates
        quant_tradedates.restype = c_int
        quant_tradedates.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_sector = quantLib.sector
        quant_sector.restype = c_int
        quant_sector.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_getdate = None
        if UtilAccess.adapter.get_os_name() == OS_Window:
            quant_getdate = quantLib.getdate
        else:
            quant_getdate = quantLib.gettradedate
        quant_getdate.restype = c_int
        quant_getdate.argtypes = [c_char_p, c_int, c_char_p, c_void_p]

        quant_csc = quantLib.csc
        quant_csc.restype = c_int
        quant_csc.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p, c_void_p]

        quant_cmc = quantLib.cmc
        quant_cmc.restype = c_int
        quant_cmc.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p, c_void_p]

        quant_chmc = quantLib.chmc
        quant_chmc.restype = c_int
        quant_chmc.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p, c_void_p]

        quant_releasedata = quantLib.releasedata
        quant_releasedata.restype = c_int
        quant_releasedata.argtypes = [c_void_p]

        quant_csq = quantLib.csq
        quant_csq.restype = c_int
        quant_csq.argtypes = [c_char_p, c_char_p, c_char_p, cls.Type_AsynDataFunc, c_void_p, c_void_p]

        quant_csqcancel = quantLib.csqcancel
        quant_csqcancel.restype = c_int
        quant_csqcancel.argtypes = [c_int]

        quant_cst = quantLib.cst
        quant_cst.restype = c_int
        quant_cst.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p, cls.Type_AsynDataFunc, c_void_p, c_void_p]

        quant_csqsnapshot = quantLib.csqsnapshot
        quant_csqsnapshot.restype = c_int
        quant_csqsnapshot.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_chq = quantLib.chq
        quant_chq.restype = c_int
        quant_chq.argtypes = [c_char_p, c_char_p, c_char_p, cls.Type_AsynDataFunc, c_void_p, c_void_p]

        quant_chqcancel = quantLib.chqcancel
        quant_chqcancel.restype = c_int
        quant_chqcancel.argtypes = [c_int]

        quant_chqsnapshot = quantLib.chqsnapshot
        quant_chqsnapshot.restype = c_int
        quant_chqsnapshot.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_ctr = quantLib.ctr
        quant_ctr.restype = c_int
        quant_ctr.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_cfc = quantLib.cfc
        quant_cfc.restype = c_int
        quant_cfc.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_cec = quantLib.cec
        quant_cec.restype = c_int
        quant_cec.argtypes = [c_char_p, c_char_p, c_void_p]

        quant_cps = quantLib.cps
        quant_cps.restype = c_int
        quant_cps.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_void_p]

        quant_setserverlistdir = quantLib.setserverlistdir
        quant_setserverlistdir.restype = None
        quant_setserverlistdir.argtypes = [c_char_p]

        quant_setproxy = quantLib.setproxy
        quant_setproxy.restype = c_int
        quant_setproxy.argtypes = [c_int, c_char_p, c_ushort, c_bool, c_char_p, c_char_p]

        quant_manualactivate = quantLib.manualactivate
        quant_manualactivate.restype = c_int
        quant_manualactivate.argtypes = [POINTER(stEQLoginInfo), c_char_p, cls.Type_logOutFunc]

        quant_setspeciallinehost = quantLib.setspeciallinehost
        quant_setspeciallinehost.restype = c_int
        quant_setspeciallinehost.argtypes = [c_char_p, c_ushort, c_bool]

        quant_setbuslinehost = quantLib.setbuslinehost
        quant_setbuslinehost.restype = c_int
        quant_setbuslinehost.argtypes = [c_char_p, c_ushort, c_bool]
		
        quant_pquery = quantLib.pquery
        quant_pquery.restype = c_int
        quant_pquery.argtypes = [c_char_p, c_void_p]

        quant_porder = quantLib.porder
        quant_porder.restype = c_int
        quant_porder.argtypes = [POINTER(stOrderInfo), c_int, c_char_p, c_char_p, c_char_p]

        quant_edb = quantLib.edb
        quant_edb.restype = c_int
        quant_edb.argtypes = [c_char_p, c_char_p, c_void_p]

        quant_edbquery = quantLib.edbquery
        quant_edbquery.restype = c_int
        quant_edbquery.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_pcreate = quantLib.pcreate
        quant_pcreate.restype = c_int
        quant_pcreate.argtypes = [c_char_p, c_char_p, c_int64, c_char_p, c_char_p]

        quant_pdelete = quantLib.pdelete
        quant_pdelete.restype = c_int
        quant_pdelete.argtypes = [c_char_p, c_char_p]

        quant_preport = quantLib.preport
        quant_preport.restype = c_int
        quant_preport.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_tradedatesnum = quantLib.tradedatesnum
        quant_tradedatesnum.restype = c_int
        quant_tradedatesnum.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_cfn = quantLib.cfn
        quant_cfn.restype = c_int
        quant_cfn.argtypes = [c_char_p, c_char_p, c_int, c_char_p, c_void_p]

        quant_cfnquery = quantLib.cfnquery
        quant_cfnquery.restype = c_int
        quant_cfnquery.argtypes = [c_char_p, c_void_p]

        quant_cnq = quantLib.cnq
        quant_cnq.restype = c_int
        quant_cnq.argtypes = [c_char_p, c_char_p, c_char_p, cls.Type_AsynDataFunc, c_void_p, c_void_p]

        quant_cnqcancel = quantLib.cnqcancel
        quant_cnqcancel.restype = c_int
        quant_cnqcancel.argtypes = [c_int]

        quant_pctransfer = quantLib.pctransfer
        quant_pctransfer.restype = c_int
        quant_pctransfer.argtypes = [c_char_p, c_char_p, c_char_p, c_double, c_char_p, c_char_p]

        quant_datastatistics = quantLib.datastatistics
        quant_datastatistics.restype = c_int
        quant_datastatistics.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        quant_secucode = quantLib.secucode
        quant_secucode.restype = c_int
        quant_secucode.argtypes = [c_char_p, c_char_p, c_char_p, c_void_p]

        ####################################

        cls.__QuantFuncDict["start"]        = quant_start
        cls.__QuantFuncDict["stop"]         = quant_stop
        cls.__QuantFuncDict["setcallback"]  = quant_setcallback
        cls.__QuantFuncDict["geterrstring"] = quant_geterrstring
        cls.__QuantFuncDict["csd"]          = quant_csd
        cls.__QuantFuncDict["css"]          = quant_css
        cls.__QuantFuncDict["cses"]         = quant_cses
        cls.__QuantFuncDict["tradedates"]   = quant_tradedates
        cls.__QuantFuncDict["sector"]       = quant_sector
        cls.__QuantFuncDict["getdate"]      = quant_getdate
        cls.__QuantFuncDict["csc"]          = quant_csc
        cls.__QuantFuncDict["cmc"]          = quant_cmc
        cls.__QuantFuncDict["chmc"]         = quant_chmc
        cls.__QuantFuncDict["releasedata"]  = quant_releasedata
        cls.__QuantFuncDict["csq"]          = quant_csq
        cls.__QuantFuncDict["csqcancel"]    = quant_csqcancel
        cls.__QuantFuncDict["cst"]          = quant_cst
        cls.__QuantFuncDict["csqsnapshot"]  = quant_csqsnapshot
        cls.__QuantFuncDict["chq"] = quant_chq
        cls.__QuantFuncDict["chqcancel"] = quant_chqcancel
        cls.__QuantFuncDict["chqsnapshot"] = quant_chqsnapshot
        cls.__QuantFuncDict["ctr"]          = quant_ctr
        cls.__QuantFuncDict["cfc"]          = quant_cfc
        cls.__QuantFuncDict["cec"]          = quant_cec
        cls.__QuantFuncDict["cps"]          = quant_cps
        cls.__QuantFuncDict["setserverlistdir"] = quant_setserverlistdir
        cls.__QuantFuncDict["setproxy"]         = quant_setproxy
        cls.__QuantFuncDict["setspeciallinehost"]= quant_setspeciallinehost
        cls.__QuantFuncDict["setbuslinehost"]   = quant_setbuslinehost
        cls.__QuantFuncDict["manualactivate"]   = quant_manualactivate
        cls.__QuantFuncDict["pquery"]   = quant_pquery
        cls.__QuantFuncDict["porder"]   = quant_porder
        cls.__QuantFuncDict["edb"]      = quant_edb
        cls.__QuantFuncDict["edbquery"] = quant_edbquery
        cls.__QuantFuncDict["pcreate"]  = quant_pcreate
        cls.__QuantFuncDict["pdelete"]  = quant_pdelete
        cls.__QuantFuncDict["preport"]  = quant_preport
        cls.__QuantFuncDict["tradedatesnum"] = quant_tradedatesnum
        cls.__QuantFuncDict["cfn"] = quant_cfn
        cls.__QuantFuncDict["cfnquery"] = quant_cfnquery
        cls.__QuantFuncDict["cnq"] = quant_cnq
        cls.__QuantFuncDict["cnqcancel"] = quant_cnqcancel
        cls.__QuantFuncDict["pctransfer"] = quant_pctransfer
        cls.__QuantFuncDict["datastatistics"] = quant_datastatistics
        cls.__QuantFuncDict["secucode"] = quant_secucode

    @classmethod
    def __Fun(cls, funcname):
        return cls.__QuantFuncDict[funcname]

    @classmethod
    def __Exec(cls, funcname, *args):
        if not cls.__InitSucceed:
            cls.__Init()
        args = list(args)
        for index in range(0, len(args)):
            if(type(args[index]) == str):
                args[index] = args[index].encode(cls.EncodeType)
        return cls.__Fun(funcname)(*args)


    @classmethod
    def start(cls, options="", logcallback=None, mainCallBack=None):
        """
        初始化和登陆(开始时调用)  options：附加参数 "TestLatency=1"
        :param options:可选参数
        :param logcallback:启动结果提示回调函数
        :return EmQuantData:
        """
        if not cls.__InitSucceed:
            cls.__Init()

        if(options.upper().find("LANGUAGEVERSION") == -1):
            options = UtilAccess.GetLanguageVersion()+","+options

        pos = options.upper().find("USERNAME=")
        uname = None
        if (pos >= 0):
            temp = options[0:pos]
            info = options[pos+9:]
            pos = info.find(",")
            if (pos > 0):
                uname = info[0:pos]
                options = temp + info[pos+1:]
            else:
                uname = info
                options = temp

        pos = options.upper().find("PASSWORD=")
        password = None
        if (pos >= 0):
            temp = options[0:pos]
            info = options[pos + 9:]
            pos = info.find(",")
            if (pos > 0):
                password = info[0:pos]
                options = temp + info[pos+1:]
            else:
                password = info
                options = temp

        optionlen = len(options)

        if(optionlen > 0 and options[optionlen-1] == ','):
            options = options[0:optionlen-1]

        data = cls.EmQuantData()

        cls.__HandleAsynDataFuncDict[0][0] = mainCallBack
        if callable(logcallback):
            cls.__logOutFunc = cls.Type_logOutFunc(logcallback)
        else:
            def log(logMessage):
                print("[EmQuantAPI Python]", logMessage.decode(cls.EncodeType))
                return 1
            cls.__logOutFunc = cls.Type_logOutFunc(log)

        cls.__Exec("setserverlistdir", _os.path.dirname(cls.__apiDllPath))
        cls.__Exec("setcallback", cls.__AsynDataFunc)

        loginResult = 0
        if (uname != None and password != None):
            loginInfo = stEQLoginInfo()
            uname = uname.encode(cls.EncodeType)
            password = password.encode(cls.EncodeType)
            loginInfo.userName = uname
            loginInfo.password = password
            loginResult = cls.__Exec("start", pointer(loginInfo), options, cls.__logOutFunc)
        else:
            loginResult = cls.__Exec("start", 0, options, cls.__logOutFunc)

        if loginResult != 0:
            data.ErrorCode = loginResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)

        return data

    @classmethod
    def stop(cls):
        """
        退出(结束时调用)
        :return: 0-成功
        """
        data = cls.EmQuantData()
        data.ErrorCode = cls.__Exec("stop")
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data

    @classmethod
    def geterrstring(cls, errcode, lang=1):
        """
        获取错误码文本说明
        :param errcode:错误代码
        :param lang:语言类型 0-中文  1-英文
        :return:
        """
        return cls.__Exec("geterrstring", errcode, lang).decode(cls.EncodeType)

    @classmethod
    def csd(cls, codes, indicators, startdate=None, enddate=None, options="", *arga, **argb):
        """
        序列数据查询(同步请求)
        :param codes: 东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param startdate:开始日期。如无分隔符，则必须为8位数字。格式支持:YYYYMMDD YYYY/MM/DD YYYY/M/D YYYY-MM-DD YYYY-M-D
        :param enddate:截止日期。如无分隔符，则必须为8位数字。格式支持:YYYYMMDD YYYY/MM/DD YYYY/M/D YYYY-MM-DD YYYY-M-D
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]

        ShowBlank = cls.__ShowBlankOption(options)
        data = cls.EmQuantData(ShowBlank)

        if(enddate == None):
            enddate = _datetime.today().strftime("%Y-%m-%d")
        if(startdate == None):
            startdate = enddate
        if(isinstance(startdate, _datetime) or isinstance(startdate, _date)):
            startdate = startdate.strftime("%Y-%m-%d")
        if(isinstance(enddate, _datetime) or isinstance(enddate, _date)):
            enddate = enddate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("csd", codes, indicators, startdate, enddate, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve3RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def css(cls, codes, indicators, options="", *arga, **argb):
        """
        截面数据查询(同步请求)
        :param codes:东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        ShowBlank = cls.__ShowBlankOption(options)

        data = cls.EmQuantData(ShowBlank)
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("css", codes, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def cses(cls, blockcodes, indicators, options="", *arga, **argb):
        """
        板块截面数据查询(同步请求)
        :param blockcodes:板块代码  多个代码间用半角逗号隔开。
        :param indicators:板块指标 多个指标间用半角逗号隔开，支持大小写。
        :param options:附加参数  多个参数以半角逗号隔开。
        :return:EmQuantData
        """
        blockcodes = cls.__toString(blockcodes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        ShowBlank = cls.__ShowBlankOption(options)

        data = cls.EmQuantData(ShowBlank)
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cses", blockcodes, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def tradedates(cls, startdate=None, enddate=None, options=None, *arga, **argb):
        """
        获取区间日期内的交易日(同步请求)
        :param startdate:开始日期。如无分隔符，则必须为8位数字。格式支持:YYYYMMDD YYYY/MM/DD YYYY/M/D YYYY-MM-DD YYYY-M-D
        :param enddate:截止日期。如无分隔符，则必须为8位数字。格式支持:YYYYMMDD YYYY/MM/DD YYYY/M/D YYYY-MM-DD YYYY-M-D
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :return EmQuantData:
        """

        if options == None:
            options = ""
        data = cls.EmQuantData()

        if(enddate == None):
            enddate = _datetime.today().strftime("%Y-%m-%d")
        if(startdate == None):
            startdate = enddate
        if(isinstance(startdate, _datetime) or isinstance(startdate, _date)):
            startdate = startdate.strftime("%Y-%m-%d")
        if(isinstance(enddate, _datetime) or isinstance(enddate, _date)):
            enddate = enddate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("tradedates", startdate, enddate, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve2RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def sector(cls, pukeycode, tradedate, options="", *arga, **argb):
        """
        获取系统板块成分(同步请求)
        :param pukeycode:
        :param tradedate:交易日
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :param arga:
        :param argb:
        :return EmQuantData:
        """

        result = cls.__PandasOptionFilter(options)
        options = result[0]
        data = cls.EmQuantData()

        if(tradedate == None):
            tradedate = _datetime.today().strftime("%Y-%m-%d")
        if(isinstance(tradedate, _datetime) or isinstance(tradedate, _date)):
            tradedate = tradedate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("sector", pukeycode, tradedate, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve2RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def getdate(cls, tradedate, offday=0, options="", *arga, **argb):
        """
        获取偏移N的交易日(同步请求)
        :param tradedate:交易日期
        :param offday:偏移天数
        :param options:
        :param arga:
        :param argb:
        :return EmQuantData:
        """
        data = cls.EmQuantData()
        if(tradedate == None):
            tradedate = _datetime.today().strftime("%Y-%m-%d")
        if(isinstance(tradedate, _datetime) or isinstance(tradedate, _date)):
            tradedate = tradedate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("getdate", tradedate, offday, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve2RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def csc(cls, code, indicators, startdate=None, enddate=None, options="", *arga, **argb):
        """
        历史分钟K线(同步请求) //code只支持单个股票
        :param code: 东财代码  支持大小写。如 "300059.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param startdate:开始日期。日期格式或日期时间14位数字年月日时分秒格式
        :param enddate:截止日期。日期格式或日期时间14位数字年月日时分秒格式
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :return EmQuantData:
        """
        code = cls.__toString(code)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        data = cls.EmQuantData()

        if (enddate == None):
            enddate = _datetime.today().strftime("%Y-%m-%d")
        if (startdate == None):
            startdate = enddate
        if (isinstance(startdate, _datetime) or isinstance(startdate, _date)):
            startdate = startdate.strftime("%Y-%m-%d")
        if (isinstance(enddate, _datetime) or isinstance(enddate, _date)):
            enddate = enddate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("csc", code, indicators, startdate, enddate, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve26RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def cmc(cls, codes, indicators, startdate=None, enddate=None, options="", *arga, **argb):
        """
        历史分钟K线(同步请求) 
        :param codes: 东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param startdate:开始日期。日期格式或日期时间14位数字年月日时分秒格式
        :param enddate:截止日期。日期格式或日期时间14位数字年月日时分秒格式
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        # broker= 有值时服务端可能返回重复代码（同一代码多条记录），需按行保留、不能用代码当字典 key 去重
        options, keepDuplicate = cls.__NeedKeepDuplicateCodes(options)
        # ispandas=0 时必须保持老的数据形态，不能把内部"按记录"的布局暴露出去
        usePandas = (result[1] != None and result[1].get("ISPANDAS") == "1")
        data = cls.EmQuantData()

        options_lower = options.lower()
        is_batch = "," in codes or "mode=batch" in options_lower

        if(enddate == None):
            enddate = _datetime.today().strftime("%Y-%m-%d")
        if(startdate == None):
            startdate = enddate
        if(isinstance(startdate, _datetime) or isinstance(startdate, _date)):
            startdate = startdate.strftime("%Y-%m-%d")
        if(isinstance(enddate, _datetime) or isinstance(enddate, _date)):
            enddate = enddate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cmc", codes, indicators, startdate, enddate, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            if keepDuplicate:
                # broker → 保留重复代码；布局沿用原来的"按指标成块"，与无 broker、异步保持一致
                data.resolve25RankData(tempData, keepDuplicateCodes=True)
                if not usePandas:
                    # 不转 pandas：还原成 Data[i][j] / Data[code][i][j] 的老形态（重复记录拼接，不丢）
                    data.applyLegacyDuplicateShape(asDict=is_batch)
            elif is_batch:
                data.resolve3RankData(tempData)  # 批量 → 和csd一样
            else:
                data.resolve26RankData(tempData)  # 单个 → 保持原有
            cls.__Exec("releasedata", pointer(tempData))

        if keepDuplicate:
            return cls.__tryResolvePandas(data, result[1], fun_name="cmcbroker")
        elif is_batch:
            return cls.__tryResolvePandas(data, result[1], fun_name="cmcbatch")
        else:
            return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def chmc(cls, codes, indicators, startdate=None, enddate=None, options="", *arga, **argb):
        """
        历史分钟K线(同步请求) 
        :param codes: 东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param startdate:开始日期。日期格式或日期时间14位数字年月日时分秒格式
        :param enddate:截止日期。日期格式或日期时间14位数字年月日时分秒格式
        :param options:附加参数  多个参数以半角逗号隔开，"Period=1,Market=CNSESH,Order=1,Adjustflag=1,Curtype=1,Pricetype=1,Type=1"
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        # broker= 有值时服务端可能返回重复代码（同一代码多条记录），需按行保留、不能用代码当字典 key 去重
        options, keepDuplicate = cls.__NeedKeepDuplicateCodes(options)
        # ispandas=0 时必须保持老的数据形态，不能把内部"按记录"的布局暴露出去
        usePandas = (result[1] != None and result[1].get("ISPANDAS") == "1")
        data = cls.EmQuantData()

        options_lower = options.lower()
        is_batch = "," in codes or "mode=batch" in options_lower

        if(enddate == None):
            enddate = _datetime.today().strftime("%Y-%m-%d")
        if(startdate == None):
            startdate = enddate
        if(isinstance(startdate, _datetime) or isinstance(startdate, _date)):
            startdate = startdate.strftime("%Y-%m-%d")
        if(isinstance(enddate, _datetime) or isinstance(enddate, _date)):
            enddate = enddate.strftime("%Y-%m-%d")
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("chmc", codes, indicators, startdate, enddate, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            if keepDuplicate:
                # broker → 保留重复代码；布局沿用原来的"按指标成块"，与无 broker、异步保持一致
                data.resolve25RankData(tempData, keepDuplicateCodes=True)
                if not usePandas:
                    # 不转 pandas：还原成 Data[i][j] / Data[code][i][j] 的老形态（重复记录拼接，不丢）
                    data.applyLegacyDuplicateShape(asDict=is_batch)
            elif is_batch:
                data.resolve3RankData(tempData)  # 批量 → 和csd一样
            else:
                data.resolve26RankData(tempData)  # 单个 → 保持原有
            cls.__Exec("releasedata", pointer(tempData))

        if keepDuplicate:
            return cls.__tryResolvePandas(data, result[1], fun_name="chmcbroker")
        elif is_batch:
            return cls.__tryResolvePandas(data, result[1], fun_name="chmcbatch")
        else:
            return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def csq(cls, codes, indicators, options="", fncallback=None, userparams=None, *arga, **argb):
        """
        实时行情(异步)  每次indicators最多为64个
        :param codes:东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param options:Pushtype=0 增量推送  1全量推送  2增量模式2(证券增量，指标全量)
        :param fncallback:不同的接口可以设定不同的回调，传NULL则使用默认的主回调函数
        :param userparams:用户参数,回调时原样返回
        :param arga:
        :param argb:
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        options = cls.__toString(options)
        data = cls.EmQuantData()

        isPkyd = any(option.replace(' ', '').upper() == "UNUSUALMOV=1" for option in options.split(','))
        isEomf = any(option.replace(' ', '').upper() == "EOMF=1" for option in options.split(','))
        callbackRequestID = 10000
        if isPkyd:
            callbackRequestID = 10004
        elif not isEomf and cls.__HasBrokerOption(options):
            callbackRequestID = 10005

        cbfunc = None
        if not callable(fncallback):
            cbfunc = DemoCallback
        else:
            cbfunc = fncallback
        cls.__HandleAsynDataFuncDict[callbackRequestID][0] = cbfunc
        ErrorCode = c_int(0)
        data.SerialID = cls.__Exec("csq", codes, indicators, options, cls.__AsynDataFunc, userparams, byref(ErrorCode))
        data.ErrorCode = ErrorCode.value
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        cls.__HandleAsynDataFuncDict[callbackRequestID][data.SerialID] = cbfunc
        if options.replace(' ', '').upper().find("ALLTICK=1") >= 0:
            cls.__setCsqSerialID.add(data.SerialID)
        return data

    @classmethod
    def csqcancel(cls, serialID):
        """
        取消实时行情订阅
        :param serialID:
        :return EmQuantData:
        """
        data = cls.EmQuantData()
        data.ErrorCode = cls.__Exec("csqcancel", serialID)
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        cls.__setCsqSerialID.discard(serialID)
        return data

    @classmethod
    def cst(cls, codes, indicators, startdatetime, enddatetime, options = "", fncallback=None, userparams=None):
        '''
        日内跳价服务(异步)  startdatetime和enddatetime格式(YYYYMMDDHHMMSS或HHMMSS表示系统日期当天的时间，两者需使用同一种格式)
        :param codes:东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param startdate:开始时间
        :param enddate:结束时间
        :param options:
        :param fncallback:不同的接口可以设定不同的回调，传NULL则使用默认的主回调函数
        :param userparams:用户参数,回调时原样返回
        :param arga:
        :param argb:
        :return EmQuantData:
        '''
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        options = cls.__toString(options)
        data = cls.EmQuantData()
        callbackRequestID = 10006 if cls.__HasBrokerOption(options) else 10001

        cbfunc = None
        if not callable(fncallback):
            cbfunc = cstCallBack
        else:
            cbfunc = fncallback
        cls.__HandleAsynDataFuncDict[callbackRequestID][0] = cbfunc
        ErrorCode = c_int(0)
        data.SerialID = cls.__Exec("cst", codes, indicators, startdatetime, enddatetime,options, cls.__AsynDataFunc, userparams, byref(ErrorCode))
        data.ErrorCode = ErrorCode.value
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        cls.__HandleAsynDataFuncDict[callbackRequestID][data.SerialID] = cbfunc
        return data

    @classmethod
    def csqsnapshot(cls, codes, indicators, options=""):
        '''
        行情快照(同步请求) 每次indicators最多为64个
        :param codes:东财代码  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "open,close,high"
        :param options:附加参数  多个参数以半角逗号隔开
        '''
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        # broker= 有值时服务端可能返回重复代码（同一代码多条记录），按重复代码逻辑保留、按日期成行
        options, keepDupExplicit = cls.__KeepDuplicateOptionFilter(options)
        isEomf = any(option.replace(' ', '').upper() == "EOMF=1" for option in options.split(','))
        isBroker = cls.__HasBrokerOption(options) and not isEomf
        if keepDupExplicit != None:
            isBroker = keepDupExplicit

        data = cls.EmQuantData()
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("csqsnapshot", codes, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData, keepDuplicateCodes=isBroker)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def chq(cls, codes, indicators, options="", fncallback=None, userparams=None, *arga, **argb):
        """
        专项服务订阅(异步)  每次indicators最多为64个
        :param codes:东财代码  多个代码间用半角逗号隔开，支持大小写。如 "515030.SH,512170.SH,515050.SH"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "TIME,PDNUM,PAMT"
        :param options:Pushtype=0 增量推送  1全量推送  2增量模式2(证券增量，指标全量)
        :param fncallback:不同的接口可以设定不同的回调，传NULL则使用默认的主回调函数
        :param userparams:用户参数,回调时原样返回
        :param arga:
        :param argb:
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        data = cls.EmQuantData()

        cbfunc = None
        if not callable(fncallback):
            cbfunc = chqDemoCallback
        else:
            cbfunc = fncallback
        cls.__HandleAsynDataFuncDict[10003][0] = cbfunc
        ErrorCode = c_int(0)
        data.SerialID = cls.__Exec("chq", codes, indicators, options, cls.__AsynDataFunc, userparams, byref(ErrorCode))
        data.ErrorCode = ErrorCode.value
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        cls.__HandleAsynDataFuncDict[10003][data.SerialID] = cbfunc
        if options.replace(' ', '').upper().find("ALLTICK=1") >= 0:
            cls.__setChqSerialID.add(data.SerialID)
        return data

    @classmethod
    def chqcancel(cls, serialID):
        """
        取消专项服务订阅
        :param serialID:
        :return EmQuantData:
        """
        data = cls.EmQuantData()
        data.ErrorCode = cls.__Exec("chqcancel", serialID)
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        cls.__setChqSerialID.discard(serialID)
        return data

    @classmethod
    def chqsnapshot(cls, codes, indicators, options=""):
        '''
        专项服务快照(同步请求) 每次indicators最多为64个
        :param codes:东财代码  多个代码间用半角逗号隔开，支持大小写。如 "515030.SH,512170.SH,515050.SH"
        :param indicators:东财指标 多个指标间用半角逗号隔开，支持大小写。如 "TIME,PDNUM,PAMT"
        :param options:附加参数  多个参数以半角逗号隔开
        '''
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        result = cls.__PandasOptionFilter(options)
        options = result[0]

        data = cls.EmQuantData()
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("chqsnapshot", codes, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def ctr(cls, ctrName, indicators="", options=""):
        '''
        获取专题报表(同步请求)
        :param ctrName: 报表名称
        :param indicator: 指标简称 多个参数以半角逗号隔开
        :param options:附加参数  多个参数以半角逗号隔开
        '''
        indicators = cls.__toString(indicators)
        data = cls.EmQuantData()

        eqData = stEQCtrData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("ctr", ctrName, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQCtrData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolveCtrData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        pos = options.upper().find("ISPANDAS=1")
        if pos >= 0 and data.ErrorCode == 0:
            import pandas as pd
            table = pd.DataFrame(data.Data, index=data.Indicators)
            table = table.T
            return table
        else:
            return data

    @classmethod
    def cfc(cls, codes, indicators, options):
        '''
        证券与指标校验函数，获取相匹配的csd/css/cses的证券和指标请求参数，并按证券品种区分
        :param codes: 证券代码
        :param indicators:指标简称
        :param options:附加参数  多个参数以半角逗号隔开，必须传入FunType=CSD 或CSS 或 CSES之一
        '''
        codes = cls.__toString(codes)
        indicators = cls.__toString(indicators)
        data = cls.EmQuantData()

        eqData = stEQCtrData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cfc", codes, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQCtrData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolveCtrData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def cec(cls, codes, options=""):
        '''
        校验或补全东财证券代码函数
        :param codes: 证券代码
        :param options:附加参数  多个参数以半角逗号隔开 ReturnType=0/1  0:返回并标记代码是否正确 1:根据SecuType与SecuMarket补全代码后缀(有可能返回多个不同的后缀)
        '''
        codes = cls.__toString(codes)
        data = cls.EmQuantData()

        eqData = stEQCtrData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cec", codes, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQCtrData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolveCtrData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def cps(cls, cpsCodes, cpsIndicators, cpsConditions, cpsOptions=''):
        '''
        条件选股函数
        :param cpsCodes  代码
        :param cpsIndicators  指标
        :param cpsConditions  条件
        :param cpsOptions  附加参数
        '''
        cpsCodes = cls.__toString(cpsCodes)
        cpsIndicators = cls.__toString(cpsIndicators)
        cpsConditions = cls.__toString(cpsConditions)

        data = cls.EmQuantData()

        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cps", cpsCodes, cpsIndicators, cpsConditions, cpsOptions, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve2RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def setserverlistdir(cls, serlistpath):
        """
        设置serverlist.json函数
        :param serlistpath 文件的路径
        """
        cls.__Exec("setserverlistdir", serlistpath)

    @classmethod
    def setproxy(cls, type, proxyip, port, verify, usr, pwd):
        """
        设置代理函数
        :param type 代理类型
        :param proxyip 代理ip
        :param port 端口
        :param verify 是否验证
        :param usr 代理用户名
        :param pwd 密码
        """
        data = cls.EmQuantData()
        coutResult = cls.__Exec("setproxy", type, proxyip, port, verify, usr, pwd)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data

    @classmethod
    def setspeciallinehost(cls, hostip, hostport, specialline):
        """
        设置专线
        :param hostip 专线地址
        :param hostport 专线端口
        :param specialline 是否设置专线
        """
        if not cls.__InitSucceed:
            cls.__Init()
        cls.__Exec("setserverlistdir", _os.path.dirname(cls.__apiDllPath))
        data = cls.EmQuantData()
        coutResult = cls.__Exec("setspeciallinehost", hostip, hostport, specialline)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data
	
    @classmethod
    def setbuslinehost(cls, hostip, hostport, busline):
        """
        设置总线
        :param hostip 总线地址
        :param hostport 总线端口
        :param busline  是否设置总线
        """
        data = cls.EmQuantData()
        coutResult = cls.__Exec("setbuslinehost", hostip, hostport, busline)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data
	
    @classmethod
    def manualactivate(cls, uname, password, options="", logcallback = None):
        """
        手动激活函数
        :param uname 用户名
        :param propasswordxyip 密码
        """
        if not cls.__InitSucceed:
            cls.__Init()
        data = cls.EmQuantData()
        loginInfo = stEQLoginInfo()
        uname = uname.encode(cls.EncodeType)
        password = password.encode(cls.EncodeType)
        loginInfo.userName = uname
        loginInfo.password = password

        appLogCallback = None
        if callable(logcallback):
            appLogCallback = cls.Type_logOutFunc(logcallback)
        else:
            def log(logMessage):
                print("[EmQuantAPI Python]", logMessage.decode(cls.EncodeType))
                return 1
            appLogCallback = cls.Type_logOutFunc(log)
        cls.__Exec("setserverlistdir", _os.path.dirname(cls.__apiDllPath))
        loginResult = cls.__Exec("manualactivate", pointer(loginInfo), options, appLogCallback)

        if loginResult != 0:
            data.ErrorCode = loginResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data

    @classmethod
    def pquery(cls, options=""):
        '''
        查询函数
        :param options 可选参数
        '''
        data = cls.EmQuantData()
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("pquery", options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def porder(cls, combincode, orderdict, remark="",options=""):
        '''
        下单函数
        :param combincode 组合代码
        :param orderdict 下单参数
        :param remark 备注
        :param options 可选参数
        '''
        if not isinstance(orderdict, dict):
            return None
        OrderMode = 0
        num = len(orderdict["code"])
        size = sizeof(stOrderInfo)
        orderinfo = (stOrderInfo*num)()
        data = cls.EmQuantData()
        volumenum = 0
        destvolumenum = 0
        weightnum = 0
        for index in range(0, num):
            for key,value in orderdict.items():
                if key == "destvolume":
                    destvolumenum = destvolumenum + 1
                elif key == "weight":
                    weightnum = weightnum + 1
                elif key == "volume":
                    volumenum = volumenum + 1
        if options.replace(' ', '').upper().find("ORDERMODE=1") >= 0:
            OrderMode = 1
            if destvolumenum != num:
                data.ErrorCode = 10003003
                data.ErrorMsg = cls.geterrstring(data.ErrorCode)
                return data
        if options.replace(' ', '').upper().find("ORDERMODE=2") >= 0:
            OrderMode = 2
            if weightnum != num:
                data.ErrorCode = 10003003
                data.ErrorMsg = cls.geterrstring(data.ErrorCode)
                return data
        if OrderMode == 0:
            if volumenum != num:
                data.ErrorCode = 10003003
                data.ErrorMsg = cls.geterrstring(data.ErrorCode)
                return data
        for index in range(0, num):
            for key,value in orderdict.items():
                if key == "code":
                    if isinstance(value[index], str):
                        setattr(orderinfo[index], key, str.encode(value[index]))
                    elif isinstance(value[index], bytes):
                        setattr(orderinfo[index], key, value[index])
                    else:
                        data.ErrorCode = 10003003
                        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
                        return data
                elif key == "volume":
                    setattr(orderinfo[index], key, value[index])
                elif key == "price":
                    setattr(orderinfo[index], key, value[index])
                elif key == "date":
                    setattr(orderinfo[index], key, int(value[index].replace("-", "").replace("/", "")))
                elif key == "time":
                    setattr(orderinfo[index], key, int(value[index].replace(":", "")))
                elif key == "optype":
                    setattr(orderinfo[index], key, value[index])
                elif key == "cost":
                    setattr(orderinfo[index], key, value[index])
                elif key == "rate":
                    setattr(orderinfo[index], key, value[index])
                elif key == "reserve":
                    setattr(orderinfo[index], key, value[index])
                elif key == "destvolume" and OrderMode == 1:
                    setattr(orderinfo[index], "volume", value[index])
                elif key == "weight" and OrderMode == 2:
                    setattr(orderinfo[index], "volume", value[index])
                else:
                    continue

        coutResult = cls.__Exec("porder", pointer(orderinfo[0]), num, combincode, remark, options)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data

    @classmethod
    def edb(cls, edbids, options):
        '''
        宏观指标服务
        '''
        edbids = cls.__toString(edbids)
        result = cls.__PandasOptionFilter(options)
        options = result[0]

        data = cls.EmQuantData()
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("edb", edbids, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve3RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def edbquery(cls, edbids, indicators="", options=""):
        '''
        宏观指标id详情查询
        '''
        edbids = cls.__toString(edbids)
        indicators = cls.__toString(indicators)

        data = cls.EmQuantData()
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("edbquery", edbids, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve3RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def pcreate(cls, combinCode, combinName, initialFound, remark, options=""):
        '''
        新建组合
        '''
        combinCode = cls.__toString(combinCode)
        data = cls.EmQuantData()
        coutResult = cls.__Exec("pcreate", combinCode, combinName, initialFound, remark, options)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        return data

    @classmethod
    def pdelete(cls, combinCode, options=""):
        '''
        删除组合
        '''
        combinCode = cls.__toString(combinCode)
        data = cls.EmQuantData()

        coutResult = cls.__Exec("pdelete", combinCode, options)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        return data

    @classmethod
    def preport(cls, combinCode, indicator, options=""):
        '''
        组合报表查询
        '''
        combinCode = cls.__toString(combinCode)
        data = cls.EmQuantData()
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("preport", combinCode, indicator, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return data

    @classmethod
    def tradedatesnum(cls, startdate, enddate, options=""):
        """
        获取区间日期内的交易日天数(同步请求)
        """
        data = cls.EmQuantData()
        if(startdate == None):
            startdate = _datetime.today().strftime("%Y-%m-%d")
        if(isinstance(startdate, _datetime) or isinstance(startdate, _date)):
            startdate = startdate.strftime("%Y-%m-%d")
        if(enddate == None):
            enddate = _datetime.today().strftime("%Y-%m-%d")
        if(isinstance(enddate, _datetime) or isinstance(enddate, _date)):
            enddate = enddate.strftime("%Y-%m-%d")
        nSumDays = c_int(0)
        refSumDays = byref(nSumDays)
        coutResult = cls.__Exec("tradedatesnum", startdate, enddate, options, refSumDays)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        data.Data = refSumDays._obj.value
        return data

    @classmethod
    def pctransfer(cls, combinCode, transferdirect, date, opCash, remark, options=""):
        """
        组合资金调配(同步请求)
         :param combinCode:组合代码
        :param transferdirect:调配方向 in 增加资金 out 减少资金
        :param date: 调配日期
        :param opCash: 增量或减少的资金量
        :param remark: 备注说明
        :param options:附加参数  多个参数以半角逗号隔开
        :return EmQuantData:
        """
        combinCode = cls.__toString(combinCode)
        if (date == None):
            date = _datetime.today().strftime("%Y-%m-%d")
        data = cls.EmQuantData()
        coutResult = cls.__Exec("pctransfer", combinCode, transferdirect, date, opCash, remark, options)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        return data

    @classmethod
    def cfn(cls, codes, content, mode, options=""):
        """
        资讯数据查询(同步请求)
        :param codes:东财代码或板块代码（不可混合）  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param content:查询内容 多个指标间用半角逗号隔开，支持大小写。如 "companynews,industrynews,report,regularreport,tradeinfo" or "sectornews"
        :param mode: 查询模式 eCfnMode_StartToEnd 和 eCfnMode_EndCount
        :param options:附加参数  多个参数以半角逗号隔开，"starttime=20190501010000,endtime=20190725,count=10"
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        content = cls.__toString(content)
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        ShowBlank = cls.__ShowBlankOption(options)

        data = cls.EmQuantData(ShowBlank)
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cfn", codes, content, mode, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankDataEx(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def cfnquery(cls, options=""):
        """
        板块树查询(同步请求)
        :param options:附加参数  多个参数以半角逗号隔开
        :return EmQuantData:
        """
        result = cls.__PandasOptionFilter(options)
        options = result[0]
        ShowBlank = cls.__ShowBlankOption(options)

        data = cls.EmQuantData(ShowBlank)
        eqData = stEQData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("cfnquery", options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolve25RankData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        return cls.__tryResolvePandas(data, result[1])

    @classmethod
    def cnq(cls, codes, content, options="", fncallback=None, userparams=None):
        """
        资讯订阅(异步)
        :param codes:东财代码或板块代码（不可混合）  多个代码间用半角逗号隔开，支持大小写。如 "300059.SZ,000002.SZ,000003.SZ,000004.SZ"
        :param content:查询内容 多个指标间用半角逗号隔开，支持大小写。如 "companynews,industrynews,report,regularreport,tradeinfo" or "sectornews"
        :param options:附加参数
        :param fncallback:不同的接口可以设定不同的回调，传NULL则使用默认的示例回调函数
        :param userparams:用户参数,回调时原样返回
        :return EmQuantData:
        """
        codes = cls.__toString(codes)
        content = cls.__toString(content)
        data = cls.EmQuantData()

        cbfunc = None
        if not callable(fncallback):
            cbfunc = cnqdemoCallBack
        else:
            cbfunc = fncallback
        cls.__HandleAsynDataFuncDict[10002][0] = cbfunc
        ErrorCode = c_int(0)
        data.SerialID = cls.__Exec("cnq", codes, content, options, cls.__AsynDataFunc, userparams, byref(ErrorCode))
        data.ErrorCode = ErrorCode.value
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        cls.__HandleAsynDataFuncDict[10002][data.SerialID] = cbfunc
        return data

    @classmethod
    def cnqcancel(cls, serialID):
        """
        取消资讯订阅
        :param serialID:
        :return EmQuantData:
        """
        data = cls.EmQuantData()
        data.ErrorCode = cls.__Exec("cnqcancel", serialID)
        data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        return data

    @classmethod
    def datastatistics(cls, funcname, indicators="", options=""):
        '''
        获取专题报表(同步请求)
        :param funcname: 单个函数名称，为空时表示查询所有
        :param indicator: 指标简称 多个参数以半角逗号隔开，为空则查询所有字段
        :param options:附加参数  多个参数以半角逗号隔开，可传STARTDATE ENDDATE
        '''
        indicators = cls.__toString(indicators)
        data = cls.EmQuantData()

        eqData = stEQCtrData()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("datastatistics", funcname, indicators, options, refEqData)
        if coutResult != 0:
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
            return data
        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQCtrData):
            data.ErrorCode = coutResult
            data.ErrorMsg = cls.geterrstring(data.ErrorCode)
        else:
            data.resolveCtrData(tempData)
            cls.__Exec("releasedata", pointer(tempData))
        pos = options.upper().find("ISPANDAS=1")
        if pos >= 0 and data.ErrorCode == 0:
            import pandas as pd
            table = pd.DataFrame(data.Data, index=data.Indicators)
            table = table.T
            return table
        else:
            return data

    @classmethod
    def secucode(cls, content, typeCodes, options=""):
        '''
        获取专题报表(同步请求)
        :param content:转换内容
        :param typeCodes:转换类型
        :param options:附加参数  多个参数以半角逗号隔开
        '''

        data = cls.EmQuantData()

        eqData = stEQChar()
        refEqData = byref(pointer(eqData))
        coutResult = cls.__Exec("secucode", content, typeCodes, options, refEqData)
        if coutResult != 0:
            err_msg = cls.geterrstring(coutResult)
            return (coutResult, err_msg)

        tempData = refEqData._obj.contents
        if not isinstance(tempData, stEQChar):
            coutResult = 10000007
            err_msg = cls.geterrstring(coutResult)
            return (coutResult, err_msg)

        result_str = ""
        if tempData.pChar is not None:
            result_str = tempData.pChar.decode(cls.EncodeType)
        cls.__Exec("releasedata", pointer(tempData))
        return (0, result_str)

    @staticmethod
    def __ShowBlankOption(options=""):
        pos = options.lower().find("showblank")
        ShowBlank = None
        if(pos >= 0):
            info = options[pos:]
            pos = info.find(",")
            if(pos > 0):
                info = info[0:pos]
            snum = info.split("=")[1]
            if(snum.isdigit() or (snum.startswith('-') and snum[1:].isdigit())):
                ShowBlank = int(snum)
        return ShowBlank

    @staticmethod
    def __PandasOptionFilter(arg=""):
        # Ispandas=1，RowIndex=1
        result_list = []
        pdDict = {}
        up_str = arg.upper()
        pos = up_str.find("ISPANDAS")
        if(pos >= 0):
            item = up_str[pos:pos+10]
            pdDict["ISPANDAS"] = item[9:10]
            arg = arg[0:pos]+arg[pos+10:]
        else:
            pdDict["ISPANDAS"] = "0"
        up_str = arg.upper()
        pos = up_str.find("ROWINDEX")
        if(pos >= 0):
            item = up_str[pos:pos+10]
            pdDict[item[0:8]] = item[9:10]
            arg = arg[0:pos]+arg[pos+10:]
        else:
            pdDict["ROWINDEX"] = "1"
        result_list.append(arg)
        result_list.append(pdDict)
        return result_list

    @staticmethod
    def __tryResolvePandas(data, args={}, fun_name=None):
        if data.ErrorCode != 0:
            return data
        if not (args != None and len(args) > 0 and "ISPANDAS" in args.keys() and "ROWINDEX" in args.keys() and args["ISPANDAS"] == "1"):
            return data

        if fun_name == None:
            import inspect
            fun_name = inspect.stack()[1][3]

        import pandas as pd

        code_list = []
        date_list = []
        data_list = []
        indictor_list = ["CODES", "DATES"]
        for ind in data.Indicators:
            data_list.append([])
        indictor_list.extend(data.Indicators)
        
        if(fun_name == "csc" or fun_name == "cmc" or fun_name == "chmc"):
            # Data 是平铺结构：Data[code_index * 指标数 + 指标下标] = 该(代码,指标)的日期序列
            nInd = len(data.Indicators)
            for code_index in range(0, len(data.Codes)):
                code = data.Codes[code_index]
                date_list.extend(data.Dates)
                for nIndex in range(0, len(data.Dates)):
                    code_list.append(code)
                base = code_index * nInd
                for j in range(0, nInd):
                    idx = base + j
                    if idx < len(data.Data):
                        data_list[j].extend(data.Data[idx])
        elif(fun_name == "cmcbroker" or fun_name == "chmcbroker"):
            # broker= 场景：Codes 可能重复，Data 按 code 下标排列（不按代码去重）
            # 每个 code 内部按指标成块：Data[code_index] = [指标0的(日期0..n), 指标1的(日期0..n), ...]
            # 取值下标：Data[code_index][indicator_index * 日期数 + date_index]
            nIndB = len(data.Indicators)
            nDateB = len(data.Dates)
            for code_index in range(0, len(data.Codes)):
                code = data.Codes[code_index]
                date_list.extend(data.Dates)
                for nIndex in range(0, len(data.Dates)):
                    code_list.append(code)
                for j in range(0, nIndB):
                    base = j * nDateB
                    data_list[j].extend(data.Data[code_index][base:base + nDateB])
        elif(fun_name == "cscbatch" or fun_name == "cmcbatch" or fun_name == "chmcbatch"):
            data.Dates = [it for it in data.Dates]
            for code_index in range(0, len(data.Codes)):
                code = data.Codes[code_index]
                date_list.extend(data.Dates)
                for nIndex in range(0, len(data.Dates)):
                    code_list.append(code)
                for cIndex in range(0, len(data.Data[code])):
                    data_list[cIndex].extend(data.Data[code][cIndex])
        elif(fun_name == "csd" or fun_name == "edb"):
            data.Dates = [_datetime.strptime(it,"%Y/%m/%d").strftime("%Y/%m/%d") for it in data.Dates]
            for code_index in range(0, len(data.Codes)):
                code = data.Codes[code_index]
                date_list.extend(data.Dates)
                for nIndex in range(0, len(data.Dates)):
                    code_list.append(code)
                for cIndex in range(0, len(data.Data[code])):
                    data_list[cIndex].extend(data.Data[code][cIndex])
        elif(fun_name == "csqsnapshot" and isinstance(data.Data, list)):
            # broker= 场景：Codes 可能重复，Data 按指标成块，与 cmc/chmc 的 broker 布局一致
            # Data[code_index][indicator_index * 日期数 + date_index]
            nIndSnap = len(data.Indicators)
            nDateSnap = len(data.Dates)
            for code_index, code in enumerate(data.Codes):
                code_list.extend([code] * len(data.Dates))
                date_list.extend(data.Dates)
                for j in range(0, nIndSnap):
                    base = j * nDateSnap
                    data_list[j].extend(data.Data[code_index][base:base + nDateSnap])
        elif(fun_name == "css" or fun_name == "cses" or fun_name == "csqsnapshot" or fun_name == "chqsnapshot"):
            for code_index in range(0, len(data.Codes)):
                code = data.Codes[code_index]
                date_list.extend(data.Dates)
                for nIndex in range(0, len(data.Dates)):
                    code_list.append(code)
                for cIndex in range(0, len(data.Data[code])):
                    data_list[cIndex].append(data.Data[code][cIndex])
        elif (fun_name == "sector"):
            indilen = len(data.Indicators)
            for code_index in range(0, len(data.Codes)):
                code = data.Codes[code_index]
                date_list.extend(data.Dates)
                for nIndex in range(0, len(data.Dates)):
                    code_list.append(code)
                for ii in range(0, indilen):
                    data_list[ii].append(data.Data[code_index*indilen+ii])
        else:
            return data

        data_list.insert(0, date_list)
        data_list.insert(0, code_list)
        table = pd.DataFrame(data_list, indictor_list)
        table = table.T

        # 重复代码场景下改稳定排序(mergesort)，保证同一代码的多条记录维持服务端返回顺序
        keepDupShape = (fun_name in ("cmcbroker", "chmcbroker")) or \
                       (fun_name == "csqsnapshot" and isinstance(data.Data, list))
        sortKind = "mergesort" if keepDupShape else "quicksort"

        if(args["ROWINDEX"] == "1"):
            table = table.sort_values(by=["CODES","DATES"], kind=sortKind).set_index(["CODES"])
        elif(args["ROWINDEX"] == "2"):
            table = table.sort_values(by=["DATES","CODES"], kind=sortKind).set_index(["DATES"])
        return table

    @staticmethod
    def __toStrArray(args):
        if(args==None or args == ""):
            return [""]
        if(isinstance(args, str)):
            return [args]
        if(isinstance(args, int) or isinstance(args, float)) :
            return [str(args)]
        if(isinstance(args, tuple) or isinstance(args, list)):
            result = []
            for item in args:
                result.extend(c.__toStrArray(item))
            return result
        return [str(args)]

    @staticmethod
    def __toNumArray(args):
        if(args == None or args == ""): return None
        if(isinstance(args, tuple)): return [int(x) for x in args]
        if(isinstance(args, list)): return [int(x) for x in args]
        if(isinstance(args, int)): return [args]
        return None

    @staticmethod
    def __toString(args, joinStr = ","):
        v = c.__toStrArray(args)
        if(v == None): return None
        return joinStr.join(v)

    @staticmethod
    def __HasBrokerOption(options):
        for option in options.split(","):
            name, separator, value = option.partition("=")
            if separator and name.strip().upper() == "BROKER" and value.strip().isdigit():
                return True
        return False

    @staticmethod
    def __KeepDuplicateOptionFilter(options):
        """
        把 keepDuplicateCodes= 从 options 中摘出来，避免透传给服务端。
        :param options: 附加参数字符串
        :return: (剩余options, 开关值)  开关值 None 表示未显式指定（由调用方按 broker 自动判断）
                 True/False 表示显式指定，优先级高于 broker 自动判断
        """
        if options == None:
            return "", None
        result = []
        value = None
        for option in options.split(","):
            name, separator, val = option.partition("=")
            if separator and name.strip().upper() == "KEEPDUPLICATECODES":
                v = val.strip().upper()
                value = (v == "1" or v == "TRUE" or v == "YES" or v == "ON")
                continue
            result.append(option)
        return ",".join(result), value

    @staticmethod
    def __NeedKeepDuplicateCodes(options):
        """
        判断本次请求是否需要保留重复代码（同一代码多条记录）。
        规则：显式 keepDuplicateCodes= 优先；未显式指定时，仅当 broker= 有值时才启用。
        """
        options, explicit = c.__KeepDuplicateOptionFilter(options)
        if explicit != None:
            return options, explicit
        return options, c.__HasBrokerOption(options)

    @staticmethod
    def __HandleAsynData(quotemessage, userparams):
        """
        实时行情回调处理函数
        :param quotemessage:
        :param userparams:
        :return:
        """
        quoteReceiveData = quotemessage.contents

        quotecallbackhandle = None
        data =c.EmQuantData()
        data.SerialID = quoteReceiveData.serialID
        data.RequestID = quoteReceiveData.requestID
        if quoteReceiveData.msgType == 0 or quoteReceiveData.msgType == 3:
            data.ErrorCode = quoteReceiveData.err
            data.ErrorMsg = c.geterrstring(data.ErrorCode)
            callbackDict = c.__HandleAsynDataFuncDict.get(quoteReceiveData.requestID, {})
            quotecallbackhandle = callbackDict.get(0)
        else:
            if quoteReceiveData.requestID in (10004, 10005, 10006):
                # 10004: csq(UnusualMov=1)   10005: csq broker=   10006: cst broker=
                # 三者都可能返回重复代码，按记录保留，不能用代码当字典 key 去重
                data.resolve25RankData(quoteReceiveData.pEQData[0], keepDuplicateCodes=True)
            elif(data.SerialID in c.__setCsqSerialID or data.SerialID in c.__setChqSerialID or quoteReceiveData.requestID == 10002):
                data.resolve25RankDataEx(quoteReceiveData.pEQData[0])
            else:
                data.resolve25RankData(quoteReceiveData.pEQData[0])
            callbackDict = c.__HandleAsynDataFuncDict.get(quoteReceiveData.requestID, {})
            quotecallbackhandle = callbackDict.get(quoteReceiveData.serialID, callbackDict.get(0))
        if not callable(quotecallbackhandle):
            quotecallbackhandle = DemoCallback
        quotecallbackhandle(data)
        return 1
